use embedded_can::{nb::Can, Frame};
use crate::emergency::{EmergencyErrorCode, ErrorRegister};

use crate::object_directory::ObjectDirectory;
use crate::pdo::PdoObjects;
use crate::prelude::*;
use crate::sdo_server::SdoState;
use crate::sdo_server::SdoState::Normal;
use crate::util::{create_frame, get_cob_id, make_abort_error};
use crate::{error, info};
use crate::error::ErrorCode;

const DEFAULT_BLOCK_SIZE: u8 = 0x7F;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum NodeState {
    Init,
    PreOperational,
    Operational,
    Stopped,
}

impl NodeState {
    pub fn heartbeat_code(&self) -> u8 {
        match *self {
            NodeState::Init => 0,
            NodeState::PreOperational => 127,
            NodeState::Operational => 5,
            NodeState::Stopped => 4,
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum NodeEvent {
    RegularTimerEvent = 1,
    NodeStart,
    Unused = 0xFF,
}

/// The implementation of Node here is not thread-safe. This implementation is
/// intended for MCU environments, where we aim for ease of use and real-time
/// performance in a single-chip environment. We have run tests on x86 as well,
/// but without enabling multi-threading.
///
/// For a thread-safe implementation, using mutexes to protect objects like
/// the Object Dictionary (OD) would be too coarse-grained. Protecting at the
/// Variable level might not incur significant performance loss due to critical
/// sections, but it still wouldn't be considered thread-safe. To achieve true
/// thread safety, we might need to replace data structures in the OD, such as
/// Map, with atomic (preferably lock-free) ones. However, currently in the
/// Rust embedded environment, there are no sufficiently mature libraries for
/// this, and we do not wish to introduce the standard (std) library, as it
/// would compromise our library's current usability in embedded environments.
pub struct Node<CAN> where CAN: Can, CAN::Frame: Frame + Debug {
    pub(crate) node_id: u8,
    pub(crate) can_network: CAN,
    pub(crate) object_directory: ObjectDirectory,
    pub(crate) pdo_objects: PdoObjects,

    // SDO specific data below:
    pub(crate) sdo_state: SdoState,
    pub(crate) read_buf: Option<Vec<u8>>,
    pub(crate) read_buf_index: usize,
    pub(crate) next_read_toggle: u8,
    pub(crate) write_buf: Option<Vec<u8>>,
    pub(crate) reserved_index: u16,
    pub(crate) reserved_sub_index: u8,
    pub(crate) write_data_size: usize,
    pub(crate) need_crc: bool,
    pub(crate) block_size: u8,
    // sequences_per_block?
    pub(crate) current_seq_number: u8,
    pub(crate) crc_enabled: bool,

    pub(crate) sync_count: u32,
    pub(crate) event_count: u32,
    pub(crate) state: NodeState,
    pub(crate) error_count: u8,
    pub(crate) heartbeats: u32,
    pub(crate) heartbeats_timer: u32,
}

impl<CAN> Node<CAN> where CAN: Can, CAN::Frame: Frame + Debug {
    pub fn new(
        node_id: u8,
        eds_content: &str,
        can_network: CAN,
    ) -> Result<Self, ErrorCode> {
        let object_directory = ObjectDirectory::new(node_id, eds_content)?;
        let pdo_objects = PdoObjects::new();
        let mut node = Node {
            node_id,
            can_network,
            object_directory,
            pdo_objects,
            sdo_state: Normal,
            read_buf: None,
            read_buf_index: 0,
            write_buf: None,
            reserved_index: 0,
            reserved_sub_index: 0,
            write_data_size: 0,
            need_crc: false,
            block_size: DEFAULT_BLOCK_SIZE,
            current_seq_number: 0,
            next_read_toggle: 0,
            crc_enabled: true,
            sync_count: 0,
            event_count: 0,
            state: NodeState::Init,
            error_count: 0,
            heartbeats: 0,
            heartbeats_timer: 0,
        };
        node.update_pdo_params()?;
        Ok(node)
    }

    pub fn pdo_objects(&self) -> &PdoObjects {
        &self.pdo_objects
    }
}

impl<CAN: Can> Node<CAN> where CAN::Frame: Frame + Debug {
    pub(crate) fn update_pdo_params(&mut self) -> Result<(), ErrorCode> {
        for i in (0x1400..0x1C00).step_by(0x200) {
            for j in 0..4 {
                let idx = i + j;

                match self.object_directory.get_variable(idx, 0) {
                    Ok(var) => {
                        let var_clone = var.clone();
                        let len: u8 = var_clone.default_value().to();
                        for k in 1..=len {
                            match self.object_directory.get_variable(idx, k) {
                                Ok(sub_var) => {
                                    let sub_var_clone = sub_var.clone();
                                    self.update(&sub_var_clone).map_err(|abort_code| {
                                        make_abort_error(abort_code, format!(
                                            "Errors in updating sub variable: {:x?}", sub_var_clone).as_str())
                                    })?;
                                }
                                Err(_) => {}
                            }
                        };
                        self.update(&var_clone).map_err(|abort_code| {
                            make_abort_error(abort_code, format!(
                                "Errors in updating variable: {:x?}", var_clone).as_str())
                        })?;
                    }
                    Err(_) => {}
                };

                let mut len = 0u8;
                let mut k = 0u8;

                while k <= len {
                    match self.object_directory.get_variable(idx, k) {
                        Ok(var) => {
                            let var_clone = var.clone();
                            self.update(&var_clone).map_err(|code| make_abort_error(
                                code, format!("Get {:?} in update var: {:x?}", code, var_clone).as_str()))?;
                            if k == 0 { len = var_clone.default_value().to(); }
                        }
                        _ => {}
                    }
                    k += 1;
                }
            }
        }
        Ok(())
    }

    pub(crate) fn filter_frame(&self, frame: &CAN::Frame) -> bool {
        if let Some(cob_id) = get_cob_id(frame) {
            if cob_id & 0x7F == self.node_id as u16 {
                return false;
            }
        }
        true
    }

    fn reset_communication(&mut self) {
        todo!();
    }
    fn reset_application(&mut self) { todo!();}
    fn reset(&mut self) {
        self.reset_communication();
        self.reset_application();

        // THINK(zephyr): Do we need to support other params, like manufacturer
        // specific?
    }

    fn process_nmt_frame(&mut self, frame: &CAN::Frame) {
        if frame.dlc() != 2 {
            return;
        }
        let (cs, nid) = (frame.data()[0], frame.data()[1]);
        info!("process_nmt_frame 1: cs = {:#x}, nid = {}", cs, nid);
        if nid != self.node_id as u8 {
            return;
        }
        match cs {
            1 => {
                info!("NMT: change state to OPERATIONAL");
                self.state = NodeState::Operational;
                self.trigger_event(NodeEvent::NodeStart);
            },
            2 => if self.state != NodeState::Init {
                info!("NMT: change state to STOPPED");
                self.state = NodeState::Stopped;
            },
            0x80 => {
                info!("NMT: change state to PRE-OPERATIONAL");
                self.state = NodeState::PreOperational
            },
            0x81 => {
                info!("NMT: change state to INIT, will reset the whole system");
                self.state = NodeState::Init;
                self.reset();
            },
            0x82 => {
                info!("NMT: change state to INIT, will reset the communication");
                self.state = NodeState::Init;
                self.reset_communication();
            },
            _ => {},
        }
    }

    // TODO(zephyr): return type to "Result<Option<CAN::Frame>, OurErr>"
    // Reorg our errors instead of String.
    fn process_rpdo_frame(&mut self, frame: &CAN::Frame) {
        if let Some(cob_id) = get_cob_id(frame) {
            if let Some(&index) = self.pdo_objects.get_rpdo_index(cob_id) {
                let rpdo = &mut self.pdo_objects.get_mut_rpdo(index);
                if frame.data().len() != ((rpdo.total_length() + 7) / 8) as usize {
                    // trigger emergency
                    let bytes = cob_id.to_le_bytes();
                    match self.trigger_emergency(
                        EmergencyErrorCode::PdoNotProcessed, ErrorRegister::GenericError, &bytes) {
                        Ok(_) => {}
                        Err(err_code) => {
                            error!("Errors in generating PdoNotProcessed EMCY frame,\
                             cod_id = {}, error_code = {:?}", cob_id, err_code);
                        }
                    }
                    return
                }
                rpdo.set_cached_data(frame.data());
            }
        }
    }

    pub fn transmit(&mut self, frame: &CAN::Frame) {
        match self.can_network.transmit(&frame) {
            Ok(_) => {
                info!("Sent frame {:x?}", frame);
            }
            Err(err) => {
                error!("Errors in transmit frame {:x?}, err: {:?}", frame, err);
            }
        }
    }

    pub fn init(&mut self) -> Result<(), ErrorCode> {
        // TODO(zephyr): this is informal, let's figure out a formal way later or just remove this.
        let ready_frame = create_frame(0x234, &[1, 2, 3, 5])?;
        Ok(self.transmit(&ready_frame))
    }

    // Need to be non-blocking.
    pub fn process_one_frame(&mut self) {
        let frame = match self.can_network.receive() {
            Ok(f) => f,
            Err(nb::Error::WouldBlock) => return,  // try next time
            Err(nb::Error::Other(err)) => {
                info!("Errors in reading CAN frame, {:?}", err);
                return
            }
        };
        info!("got frame: {:?}", frame);
        if let Some(cob_id) = get_cob_id(&frame) {
            match cob_id & 0xFF80 {
                0x000 => self.process_nmt_frame(&frame),
                0x200..=0x500 => self.process_rpdo_frame(&frame),
                0x080 => self.process_sync_frame(),
                0x600 => self.process_sdo_frame(&frame),
                _ => {},
            }
        }
    }

    pub fn trigger_event(&mut self, event: NodeEvent) {
        match event {
            NodeEvent::NodeStart => {
                self.event_count = 0;
                self.sync_count = 0;
                self.error_count = 0;
                self.heartbeats = 0;
                self.transmit_pdo_messages(false, event, self.event_count);
            }
            _ => {}
        }
    }

    fn process_sync_frame(&mut self) {
        if self.state == NodeState::Operational {
            self.sync_count += 1;
            self.save_rpdo_messages(true, NodeEvent::Unused, self.sync_count);
            self.transmit_pdo_messages(true, NodeEvent::Unused, self.sync_count);
        }
    }

    pub fn event_timer_callback(&mut self) {
        // info!("event_timer_callback 0, state = {:?}", self.state);
        if self.heartbeats_timer > 0 {
            self.heartbeats += 1;
            if self.heartbeats % self.heartbeats_timer == 0 {
                match create_frame(0x700 + self.node_id as u16, &[self.state.heartbeat_code()]) {
                    Ok(frame) => { self.transmit(&frame) }
                    Err(ec) => {
                        error!("Errors in creating heartbeat CAN frame: error_code = {:?}", ec);
                    }
                }
            }
        }

        if self.state == NodeState::Operational {
            self.event_count += 1;
            self.save_rpdo_messages(false, NodeEvent::RegularTimerEvent, self.event_count);
            self.transmit_pdo_messages(false, NodeEvent::RegularTimerEvent, self.event_count);
        }
    }
}
