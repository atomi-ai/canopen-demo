use embedded_can::Frame;
use embedded_can::nb::Can;

use crate::cmd_header::{
    SdoBlockDownloadInitiateCmd, SdoBlockUploadCmd, SdoDownloadInitiateCmd, SdoDownloadSegmentCmd,
    SdoEndBlockDownloadCmd, SdoInitBlockUploadCmd,
};
use crate::constant::{COB_FUNC_TRANSMIT_SDO, REG_PRODUCER_HEARTBEAT_TIME, REG_RESTORE_DEFAULT_PARAMETERS, RESET_MAGIC_CODE};
use crate::error::ErrorCode;
use crate::error;
use crate::error::AbortCode::{CommandSpecifierNotValidOrUnknown, DataTransferOrStoreFailed, GeneralError, InvalidBlockSize, ObjectCannotBeMappedToPDO, ToggleBitNotAlternated};
use crate::node::Node;
use crate::prelude::*;
use crate::sdo_server::SdoState::{ConfirmUploadSdoBlock, DownloadSdoBlock, EndSdoBlockDownload, FinalConfirmUploadSdoBlock, Normal, SdoSegmentDownload, SdoSegmentUpload, StartSdoBlockUpload};
use crate::util::{convert_bytes_to_u32, crc16_canopen_with_lut, create_frame_with_padding, flatten, make_abort_error};

/// Represents the various states of the SDO (Service Data Object) communication process.
/// These states govern the different phases or modes of SDO transmissions in a CANopen system.
pub enum SdoState {
    /// The default state of SDO communication where no specific SDO operation is active.
    Normal,

    /// The state when segments of data are being uploaded from the server to the client.
    SdoSegmentUpload,

    /// The state when segments of data are being downloaded from the client to the server.
    SdoSegmentDownload,

    /// The state when blocks of data are being downloaded in a block-wise manner from the client to the server.
    DownloadSdoBlock,

    /// The state marking the conclusion of a block download process.
    EndSdoBlockDownload,

    /// The state marking the beginning of a block upload process.
    StartSdoBlockUpload,

    /// The state where the server waits for the client's confirmation after uploading blocks of data.
    ConfirmUploadSdoBlock,
    FinalConfirmUploadSdoBlock,
}

impl<CAN: Can> Node<CAN> where CAN::Frame: Frame + Debug {
    fn create_can_frame(&self, data: &[u8]) -> Result<Option<CAN::Frame>, ErrorCode> {
        create_frame_with_padding(COB_FUNC_TRANSMIT_SDO | self.node_id as u16, data)
            .map(Some)
            .map_err(|ec| make_abort_error(GeneralError, format!("{:?}", ec)))
    }

    fn create_sdo_frame(&self, cmd: u8, index: u16, sub_index: u8, data: &[u8])
                        -> Result<Option<CAN::Frame>, ErrorCode> {
        let bytes = flatten(&[&[cmd], &index.to_le_bytes(), &sub_index.to_le_bytes(), data]);
        self.create_can_frame(&bytes)
    }

    fn next_state(&mut self, state: SdoState, res: Result<Option<CAN::Frame>, ErrorCode>)
                  -> Result<Option<CAN::Frame>, ErrorCode> {
        self.sdo_state = state;
        res
    }

    pub(crate) fn process_sdo_frame(&mut self, frame: &CAN::Frame) {
        if self.filter_frame(frame) {
            return;
        }
        let cmd = frame.data()[0];
        let ccs = cmd >> 5;

        let index = u16::from_le_bytes([frame.data()[1], frame.data()[2]]);
        let sub_index = frame.data()[3];
        let res = match &self.sdo_state {
            SdoSegmentDownload => self.download_segment(frame.data()),
            SdoSegmentUpload => self.upload_segment(cmd),
            DownloadSdoBlock => self.block_download(frame.data()),
            EndSdoBlockDownload => self.end_block_download(frame.data()),
            StartSdoBlockUpload => self.start_block_upload(frame.data()),
            ConfirmUploadSdoBlock => self.confirm_block_upload(frame.data()),
            FinalConfirmUploadSdoBlock => self.final_confirm_block_upload(frame.data()),
            Normal => {
                // ccs: 0x1 / 0x2 / 0x6 / 0x5, based on Canopen 301.
                match ccs {
                    0x1 => self.initiate_download(index, sub_index, frame.data()),
                    0x2 => self.initiate_upload(index, sub_index),
                    0x6 => self.init_block_download(index, sub_index, frame.data()),
                    0x5 => self.init_block_upload(index, sub_index, frame.data()),
                    _ => Err(make_abort_error(CommandSpecifierNotValidOrUnknown, "".to_string())),
                }
            }
        };

        match res {
            Ok(resp) => {
                if let Some(frame) = resp {
                    self.transmit(&frame);
                }
            }
            Err(ErrorCode::AbortCodeWrapper { abort_code, .. }) => {
                if abort_code == GeneralError {
                    error!("debug");
                }
                let (idx, sidx) = match self.sdo_state {
                    Normal => (index, sub_index),
                    _ => (self.reserved_index, self.reserved_sub_index),
                };
                self.sdo_state = Normal;
                self.read_buf = None;
                self.write_buf = None;
                self.need_crc = false;

                match self.create_sdo_frame(0x80, idx, sidx, &abort_code.code().to_le_bytes()) {
                    Ok(Some(err_frame)) => { self.transmit(&err_frame) }
                    Err(err) => {
                        error!("Errors in creating SDO abort frame, index = {},\
                         sub_index = {}, abort_code = {:x?}, err: {:?}", idx, sidx, abort_code, err);
                    }
                    _ => {}
                }
            }
            Err(err) => {
                error!("Errors in processing SDO frame: {:x?}, err: {:?}", frame, err);
            }
        }
    }

    fn initiate_upload(&mut self, index: u16, sub_index: u8) -> Result<Option<CAN::Frame>, ErrorCode> {
        let var = self.object_directory.get_variable(index, sub_index)?;
        let data = var.default_value().data();

        if data.is_empty() {
            return Err(make_abort_error(GeneralError, "".to_string()));
        }

        if data.len() <= 4 {
            let cmd = 0x43 | ((((4 - data.len()) as u8) & 0x3) << 2);
            let t = data.clone();
            return self.create_sdo_frame(cmd, index, sub_index, &t);
        }

        self.read_buf = Some(data.clone());
        self.read_buf_index = 0;
        self.next_read_toggle = 0;
        self.reserved_index = index;
        self.reserved_sub_index = sub_index;
        let len_bytes_vec = (data.len() as u32).to_le_bytes().to_vec();
        let res = self.create_sdo_frame(0x41, index, sub_index, &len_bytes_vec);
        self.next_state(SdoSegmentUpload, res)
    }

    fn upload_segment(&mut self, cmd: u8) -> Result<Option<CAN::Frame>, ErrorCode> {
        // Check if the command specifier is correct for an upload segment.
        if cmd >> 5 != 0x3 {
            return Err(make_abort_error(GeneralError, "".to_string()));
        }

        // Ensure the read buffer is available and has data to send.
        let buffer = self.read_buf.as_mut().ok_or(make_abort_error(GeneralError, "".to_string()))?;
        let toggle = (cmd >> 4) & 0x1;

        // Check the toggle bit for proper alternating value.
        if toggle != self.next_read_toggle {
            return Err(make_abort_error(ToggleBitNotAlternated, "".to_string()));
        }

        // Prepare for the next toggle.
        self.next_read_toggle ^= 1;
        let remaining_data = &buffer[self.read_buf_index..];
        let remaining_len = remaining_data.len();

        // If more than 7 bytes of data remain, send the next 7-byte segment.
        if remaining_len > 7 {
            self.read_buf_index += 7;
            let data = [&[toggle << 4], &remaining_data[..7]].concat();
            self.create_can_frame(&data)
        } else {
            // Handle the remaining data, setting 'n' for the number of unused bytes and 'c=1' for end of segment.
            let n = 7 - remaining_len as u8;
            let data = [&[0x01 | (toggle << 4) | (n << 1)], remaining_data].concat();
            self.read_buf = None;
            self.read_buf_index = 0;

            // Transition to the Normal state after the last segment.
            self.next_state(Normal, self.create_can_frame(&data))
        }
    }

    fn initiate_download(&mut self, index: u16, sub_index: u8, req: &[u8]) -> Result<Option<CAN::Frame>, ErrorCode> {
        let cmd = SdoDownloadInitiateCmd::from(req[0]);

        // Check if the download is expedited.
        if cmd.e() && cmd.s() {
            // Handle expedited download.
            let data = &req[4..(8 - cmd.n() as usize)];
            self.set_value_with_check(index, sub_index, data)?;
            return self.create_sdo_frame(0x60, index, sub_index, &[0, 0, 0, 0]);
        }

        // Set up for normal download.
        self.write_buf = Some(Vec::new());
        self.reserved_index = index;
        self.reserved_sub_index = sub_index;

        // Determine the write data size.
        self.write_data_size = if cmd.s() {
            convert_bytes_to_u32(&req[4..])? as usize
        } else {
            0
        };

        // Create and send the response frame for normal download initiation.
        let response = self.create_sdo_frame(0x60, index, sub_index, &[0, 0, 0, 0]);
        self.next_state(SdoSegmentDownload, response)
    }

    fn download_segment(&mut self, req: &[u8]) -> Result<Option<CAN::Frame>, ErrorCode> {
        let req_cmd = SdoDownloadSegmentCmd::from(req[0]);
        if req_cmd.ccs() != 0x0 {
            return Err(make_abort_error(GeneralError, "".to_string()));
        }

        let mut buf = self.write_buf.take().ok_or(make_abort_error(GeneralError, "".to_string()))?;
        let result = (|| {
            let resp_cmd = 0x20 | (req_cmd.t() << 4);
            if !req_cmd.c() {
                // Not finished, append data and continue.
                buf.extend_from_slice(&req[1..]);
                self.create_can_frame(&[resp_cmd])
            } else {
                // Handle the final segment of the download.
                buf.extend_from_slice(&req[1..(8 - req_cmd.n() as usize)]);
                if self.write_data_size > 0 && self.write_data_size != buf.len() {
                    return Err(make_abort_error(GeneralError, "".to_string())); // Size mismatch error.
                }
                let (index, sub_index) = (self.reserved_index, self.reserved_sub_index);
                self.set_value_with_check(index, sub_index, &buf)?;
                self.create_can_frame(&[resp_cmd])
            }
        })();
        // Regardless of the outcome, restore the write_buf.
        self.write_buf = Some(buf);
        self.next_state(Normal, result)
    }

    fn init_block_download(&mut self, index: u16, sub_index: u8, req: &[u8]) -> Result<Option<CAN::Frame>, ErrorCode> {
        let cmd = SdoBlockDownloadInitiateCmd::from(req[0]);

        // Update the flag for CRC need based on the command.
        self.need_crc = cmd.cc();

        // Determine the write data size if specified, otherwise set it to zero.
        self.write_data_size = if cmd.s() {
            convert_bytes_to_u32(&req[4..8])? as usize
        } else {
            0
        };

        // Initialize the buffer for block download and set other related parameters.
        self.write_buf = Some(Vec::new());
        self.current_seq_number = 0;
        self.reserved_index = index;
        self.reserved_sub_index = sub_index;

        // Create the response frame for initiating block download.
        let resp_cmd = 0xA0 | ((self.crc_enabled as u8) << 2);
        let response_payload = [self.block_size, 0, 0, 0];
        let response = self.create_sdo_frame(resp_cmd, index, sub_index, &response_payload);

        // Transition to the DownloadSdoBlock state.
        self.next_state(DownloadSdoBlock, response)
    }

    fn block_download(&mut self, req: &[u8]) -> Result<Option<CAN::Frame>, ErrorCode> {
        let seqno = req[0] & 0x7F;
        self.current_seq_number += 1;
        if seqno != self.current_seq_number {
            return Err(make_abort_error(GeneralError, "".to_string()));
        }

        let mut buf = self.write_buf.take().ok_or(make_abort_error(GeneralError, "".to_string()))?;
        buf.extend_from_slice(&req[1..]);

        let result = (|| {
            if req[0] >> 7 == 1 {
                // No more segments
                if buf.len() >= self.write_data_size && buf.len() - 7 < self.write_data_size {
                    buf.resize(self.write_data_size, 0);
                }
                // TODO(zephyr): Check correctness: CRC

                // Write data to object directory.
                let (i, si) = (self.reserved_index, self.reserved_sub_index);
                self.set_value_with_check(i, si, &buf)?;

                let (c, b) = (self.current_seq_number, self.block_size);
                self.next_state(EndSdoBlockDownload, self.create_can_frame(&[0xA2, c, b]))
            } else {
                self.create_can_frame(&[])
            }
        })();
        self.write_buf = Some(buf);
        result
    }

    fn end_block_download(&mut self, req: &[u8]) -> Result<Option<CAN::Frame>, ErrorCode> {
        let cmd = SdoEndBlockDownloadCmd::from(req[0]);
        if cmd.n() as usize != 7 - self.write_data_size % 7 {
            return Err(make_abort_error(GeneralError, "".to_string()));
        }
        // TODO(zephyr): CRC check.
        let _crc = u16::from_le_bytes([req[1], req[2]]);

        self.next_state(Normal, self.create_can_frame(&[0xA1]))
    }

    fn init_block_upload(&mut self, index: u16, sub_index: u8, req: &[u8])
        -> Result<Option<CAN::Frame>, ErrorCode> {
        let cmd = SdoInitBlockUploadCmd::from(req[0]);
        let (blk_size, _pst) = (req[4], req[5]);

        if cmd.ccs() != 0x5 || cmd.cs() != 0 {
            return Err(make_abort_error(GeneralError, "".to_string()));
        }
        if blk_size >= 0x80 {
            return Err(make_abort_error(InvalidBlockSize, "".to_string()));
        }

        // Init setting for upload (read)
        self.need_crc = cmd.cc();
        self.block_size = blk_size;
        self.reserved_index = index;
        self.reserved_sub_index = sub_index;
        let var = self.object_directory.get_variable(index, sub_index)?;
        self.read_buf = Some(var.default_value().data().clone());
        self.read_buf_index = 0;

        // Prepare the response packet.
        let resp_cmd = 0xC2 | (self.crc_enabled as u8) << 2;
        let v: [u8; 4] = (self.read_buf.as_ref()
            .ok_or(make_abort_error(GeneralError, "".to_string()))?.len() as u32)
            .to_le_bytes();
        let res = self.create_sdo_frame(resp_cmd, index, sub_index, &v);
        self.next_state(StartSdoBlockUpload, res)
    }

    fn start_block_upload(&mut self, req: &[u8]) -> Result<Option<CAN::Frame>, ErrorCode> {
        let cmd = SdoBlockUploadCmd::from(req[0]);
        if cmd.ccs() != 0x5 || cmd.cs() != 0x3 {
            return Err(make_abort_error(GeneralError, "".to_string()));
        }

        // TODO(zephyr): Some additional scenarios that need consideration:
        // - total_seqs > blksize: Transmission requires multiple blocks, each
        //   containing several sequences.
        // - Logic for retransmission based on ack seq. (This also needs to be
        //   considered for download)

        let buf = self.read_buf.take().ok_or(make_abort_error(GeneralError, "".to_string()))?;
        let result = (|| {
            let total_seqs = ((buf.len() - 1) / 7 + 1) as u8;
            for i in 0..total_seqs - 1 {
                // This is a special case, directly transmit (total_seq - 1) frames,
                // only leave the last one at last for change the state.
                let (s, e) = ((i * 7) as usize, (i * 7 + 7) as usize);
                let bytes = [&[i + 1], &buf[s..e]].concat();
                let frame = create_frame_with_padding(COB_FUNC_TRANSMIT_SDO | self.node_id as u16, &bytes)?;
                self.transmit(&frame);
            }
            let s = ((total_seqs - 1) * 7) as usize;
            self.create_can_frame(flatten(&[&[total_seqs | 0x80], &buf[s..]]).as_slice())
        })();
        self.read_buf = Some(buf);

        self.next_state(ConfirmUploadSdoBlock, result)
    }

    fn confirm_block_upload(&mut self, req: &[u8]) -> Result<Option<CAN::Frame>, ErrorCode> {
        let cmd = SdoBlockUploadCmd::from(req[0]);
        if cmd.ccs() != 0x5 || cmd.cs() != 2 {
            return Err(make_abort_error(GeneralError, "".to_string()));
        }
        let buf = self.read_buf.as_ref().ok_or(make_abort_error(GeneralError, "".to_string()))?;
        let (ackseq, blksize) = (req[1], req[2]);
        if ackseq as usize != (buf.len() - 1) / 7 + 1 {
            return Err(make_abort_error(CommandSpecifierNotValidOrUnknown, "".to_string()));
        }

        self.block_size = blksize;
        let n = (7 - buf.len() % 7) as u8;
        let resp_cmd = 0xC1 | (n << 2);
        let crc: u16 = if self.need_crc {
            crc16_canopen_with_lut(buf)
        } else {
            0
        };
        let mut response_data = vec![resp_cmd];
        response_data.extend_from_slice(&crc.to_le_bytes());
        response_data.extend([0, 0, 0, 0, 0]);
        self.next_state(FinalConfirmUploadSdoBlock, self.create_can_frame(&response_data))
    }

    fn final_confirm_block_upload(&mut self, req: &[u8]) -> Result<Option<CAN::Frame>, ErrorCode> {
        let cmd = SdoBlockUploadCmd::from(req[0]);
        if cmd.ccs() != 0x5 || cmd.cs() != 1 {
            return Err(make_abort_error(GeneralError, "".to_string()));
        }
        self.next_state(Normal, Ok(None))
    }

    fn validate_pdo_mapping_params_on_setting(&mut self, index: u16, sub_index: u8, data: &[u8])
                                              -> Result<(), ErrorCode> {
        // Early return if sub_index is not within valid PDO mapping range.
        if !(sub_index > 0 && sub_index <= crate::pdo::MAX_PDO_MAPPING_LENGTH) {
            return Ok(());
        }

        // Check data length is as expected for PDO mapping.
        if data.len() != 4 {
            return Err(make_abort_error(ObjectCannotBeMappedToPDO, "".to_string()));
        }

        // Extract destination index and sub-index from data.
        let dest_index = (data[3] as u16) << 8 | (data[2] as u16);
        let dest_sub_index = data[1];

        // Retrieve variable and validate if it can be mapped to PDO.
        let var = self.object_directory.get_variable(dest_index, dest_sub_index)?;
        if !var.pdo_mappable() || (index < 0x1800 && !var.access_type().is_writable()) {
            return Err(make_abort_error(ObjectCannotBeMappedToPDO, "".to_string()));
        }

        Ok(())
    }

    fn try_reset(&mut self, sub_index: u8, data: &[u8]) -> Result<bool, ErrorCode> {
        let magic_code = u32::from_le_bytes(data[0..4].try_into().map_err(|_| {
            error!("Errors in converting data: {:x?}", data);
            make_abort_error(GeneralError, "".to_string())
        })?);
        if magic_code != RESET_MAGIC_CODE {
            return Err(make_abort_error(DataTransferOrStoreFailed, "".to_string()));
        }
        let success = match sub_index {
            0x1 => self.reset(),
            0x2 => self.reset_communication(),
            0x3 => self.reset_application(),
            _ => { false }
        };

        if success {
            self.object_directory.set_value(REG_RESTORE_DEFAULT_PARAMETERS, sub_index, &1u32.to_le_bytes(), true)?;
        }

        Ok(true)
    }

    fn set_value_preprocess(&mut self, index: u16, sub_index: u8, data: &[u8]) -> Result<bool, ErrorCode> {
        match index {
            0x1600..=0x17FF | 0x1A00..=0x1BFF =>
                self.validate_pdo_mapping_params_on_setting(index, sub_index, data).map(|_| false),
            REG_RESTORE_DEFAULT_PARAMETERS => self.try_reset(sub_index, data),
            _ => Ok(false),
        }
    }

    fn set_value_with_check(&mut self, index: u16, sub_index: u8, data: &[u8]) -> Result<(), ErrorCode> {
        if self.set_value_preprocess(index, sub_index, data)? {
            return Ok(());
        }
        let var = self.object_directory.set_value(index, sub_index, data, false)?;
        match index {
            0x1400..=0x1BFF => {
                let var_clone = var.clone();
                self.update(&var_clone)?;
            }
            REG_PRODUCER_HEARTBEAT_TIME => {
                let t: u16 = var.default_value().to();
                self.heartbeats_timer = t as u32;
            }
            _ => {}
        }
        Ok(())
    }
}
