use std::thread;
use std::time::Duration;
use log::{debug, info};
use socketcan::CanSocket;
use canopen_rust::error::ErrorCode;
use crate::async_util::AsyncExpectorOld;
use crate::util::{exp, send};

pub fn tpdo_event_driven_mode(s: &CanSocket, interface_name: &'static str) -> Result<(), ErrorCode> {
    // Set state to pre-operational and set 0x1800 related.
    send(&s, 0x000, 0x80_02, 2);
    // Write value C0000184h to object 1800h:01h
    send(&s, 0x602, 0x23_00_18_01_82_01_00_C0, 8);
    exp(&s, 0x582, 0x60_00_18_01_00_00_00_00, 8);
    // Write value FEh to object 1800h:02h
    send(&s, 0x602, 0x2F_00_18_02_FE_00_00_00, 8);
    exp(&s, 0x582, 0x60_00_18_02_00_00_00_00, 8);
    // Write value 0Ah to object 1800h:05h
    send(&s, 0x602, 0x2B_00_18_05_0A_00_00_00, 8);
    exp(&s, 0x582, 0x60_00_18_05_00_00_00_00, 8);
    // Write value 40000184h to object 1800h:01h
    send(&s, 0x602, 0x23_00_18_01_82_01_00_40, 8);
    exp(&s, 0x582, 0x60_00_18_01_00_00_00_00, 8);

    // Write value 40000284h to object 1801h:01h, enable PDO on 0x282
    send(&s, 0x602, 0x23_01_18_01_82_02_00_40, 8);
    exp(&s, 0x582, 0x60_01_18_01_00_00_00_00, 8);

    send(&s, 0x602, 0x40_00_60_01_00_00_00_00, 8);
    exp(&s, 0x582, 0x4F_00_60_01_FF_00_00_00, 8);

    {
        // Need to scope the ExpectContainer.
        let mut ec = AsyncExpectorOld::new(interface_name);
        // Set device to Operational
        send(&s, 0x000, 0x01_02, 2);
        for _ in 0..3 {
            ec.async_expect(0x182, 0xFF_00, 2)?;
        }
        ec.async_expect(0x282, 0xFF_03_FF_03_FF_03_FF_03, 8)?;

        let res = ec.wait_for_all();
        info!("xfguo: result of wait_for_all(): res = '{}', len = {}", res, res.len());
        assert_eq!(res.as_str(), "");
    }

    info!("Let server into pre-operational state");
    // Set device to Pre-Operational
    send(&s, 0x000, 0x80_02, 2);

    // Write value C0000282h to object 1801h:01h, disable PDO on 0x282
    send(&s, 0x602, 0x23_01_18_01_82_02_00_C0, 8);
    exp(&s, 0x582, 0x60_01_18_01_00_00_00_00, 8);
    // Write value FEh to object 1801h:02h
    send(&s, 0x602, 0x2F_01_18_02_FE_00_00_00, 8);
    exp(&s, 0x582, 0x60_01_18_02_00_00_00_00, 8);
    // PDO 04: Write value 0Ah to object 1801h:05h
    send(&s, 0x602, 0x2B_01_18_05_0A_00_00_00, 8);
    exp(&s, 0x582, 0x60_01_18_05_00_00_00_00, 8);
    // PDO 04: Write value 40000284h to object 1801h:01h, enable PDO on 0x282
    send(&s, 0x602, 0x23_01_18_01_82_02_00_40, 8);
    exp(&s, 0x582, 0x60_01_18_01_00_00_00_00, 8);
    {
        // Need to scope the ExpectContainer.
        let mut ec = AsyncExpectorOld::new(interface_name);
        // Set device to Operational
        send(&s, 0x000, 0x01_02, 2);
        for _ in 0..3 {
            ec.async_expect(0x182, 0xFF_00, 2)?;
            ec.async_expect(0x282, 0xFF_03_FF_03_FF_03_FF_03, 8)?;
        }
        let res = ec.wait_for_all();
        info!("xfguo: result of wait_for_all(): res = '{}', len = {}", res, res.len());
        assert_eq!(res.as_str(), "");
    }

    // Set device to Pre-Operational, stop the TPDO service.
    info!("Let server into pre-operational state");
    send(&s, 0x000, 0x80_02, 2);

    Ok(())
}

pub fn tpdo_sync_mode(s: &CanSocket, interface_name: &'static str) -> Result<(), ErrorCode> {
    // Set state to pre-operational and set 0x1800 related.
    send(&s, 0x000, 0x80_02, 2);
    // Write value C0000184h to object 1800h:01h
    send(&s, 0x602, 0x23_00_18_01_82_01_00_C0, 8);
    exp(&s, 0x582, 0x60_00_18_01_00_00_00_00, 8);
    // Write value 0Ah to object 1800h:02h
    send(&s, 0x602, 0x2F_00_18_02_0A_00_00_00, 8);
    exp(&s, 0x582, 0x60_00_18_02_00_00_00_00, 8);
    // Write value 40000184h to object 1800h:01h, enable PDO on 0x182
    send(&s, 0x602, 0x23_00_18_01_82_01_00_40, 8);
    exp(&s, 0x582, 0x60_00_18_01_00_00_00_00, 8);

    // Write value 40000284h to object 1801h:01h, enable PDO on 0x282
    send(&s, 0x602, 0x23_01_18_01_82_02_00_40, 8);
    exp(&s, 0x582, 0x60_01_18_01_00_00_00_00, 8);

    {
        // Need to scope the ExpectContainer.
        let mut ec = AsyncExpectorOld::new(interface_name);
        // Set device to Operational
        send(&s, 0x000, 0x01_02, 2);
        ec.async_expect(0x282, 0xFF_03_FF_03_FF_03_FF_03, 8)?;
        ec.unexpect(0x182, 0xFF_00, 2)?;
        for _ in 0..3 {
            for _ in 0..10 { ec.send(0x80, 0x0, 0)?; }
            ec.expect(0x182, 0xFF_00, 2)?;
        }
        let res = ec.wait_for_all();
        assert_eq!(res.as_str(), "");
    }

    // Test Notice	: PDO 04: Set device to Pre-Operational
    send(&s, 0x000, 0x80_02, 2);
    // Write value C0000282h to object 1801h:01h, disable PDO on 0x282
    send(&s, 0x602, 0x23_01_18_01_82_02_00_C0, 8);
    exp(&s, 0x582, 0x60_01_18_01_00_00_00_00, 8);
    // Write value FEh to object 1801h:02h
    send(&s, 0x602, 0x2F_01_18_02_0A_00_00_00, 8);
    exp(&s, 0x582, 0x60_01_18_02_00_00_00_00, 8);
    // Write value 40000284h to object 1801h:01h, enable PDO on 0x282
    send(&s, 0x602, 0x23_01_18_01_82_02_00_40, 8);
    exp(&s, 0x582, 0x60_01_18_01_00_00_00_00, 8);
    {
        // Need to scope the ExpectContainer.
        let mut ec = AsyncExpectorOld::new(interface_name);
        // Set device to Operational
        send(&s, 0x000, 0x01_02, 2);
        for _ in 0..3 {
            for _ in 0..10 { ec.send(0x80, 0x0, 0)?; }
            ec.expect(0x182, 0xFF_00, 2)?;
            ec.expect(0x282, 0xFF_03_FF_03_FF_03_FF_03, 8)?;
        }
        let res = ec.wait_for_all();
        assert_eq!(res.as_str(), "");
    }

    // Set device to Pre-Operational, stop the TPDO service.
    send(&s, 0x000, 0x80_02, 2);
    Ok(())
}

pub fn rpdo_event_driven_mode(s: &CanSocket) ->Result<(), ErrorCode> {
    // Disable TPDOs
    send(&s, 0x602, 0x23_00_18_01_82_01_00_C0, 8);
    exp(&s, 0x582, 0x60_00_18_01_00_00_00_00, 8);
    send(&s, 0x602, 0x23_01_18_01_82_02_00_C0, 8);
    exp(&s, 0x582, 0x60_01_18_01_00_00_00_00, 8);

    // Set state to pre-operational and set 0x1800 related.
    send(&s, 0x000, 0x80_02, 2);
    // Write value C0000202h to object 1400h:01h
    send(&s, 0x602, 0x23_00_14_01_02_02_00_C0, 8);
    exp(&s, 0x582, 0x60_00_14_01_00_00_00_00, 8);
    // Write value FEh to object 1400h:02h
    send(&s, 0x602, 0x2F_00_14_02_FE_00_00_00, 8);
    exp(&s, 0x582, 0x60_00_14_02_00_00_00_00, 8);
    // Write value 64h to object 1400h:05h
    send(&s, 0x602, 0x2B_00_14_05_0A_00_00_00, 8);
    exp(&s, 0x582, 0x60_00_14_05_00_00_00_00, 8);
    // Write value 40000202h to object 1400h:01h, enable the RPDO object
    send(&s, 0x602, 0x23_00_14_01_02_02_00_40, 8);
    exp(&s, 0x582, 0x60_00_14_01_00_00_00_00, 8);

    send(&s, 0x602, 0x40_00_16_00_00_00_00_00, 8);
    exp(&s, 0x582, 0x4F_00_16_00_02_00_00_00, 8);
    send(&s, 0x602, 0x40_00_16_01_00_00_00_00, 8);
    exp(&s, 0x582, 0x43_00_16_01_08_01_00_62, 8);
    send(&s, 0x602, 0x40_00_16_02_00_00_00_00, 8);
    exp(&s, 0x582, 0x43_00_16_02_08_02_00_62, 8);

    send(&s, 0x602, 0x2F_00_62_01_00_00_00_00, 8);
    exp(&s, 0x582, 0x60_00_62_01_00_00_00_00, 8);
    send(&s, 0x602, 0x2F_00_62_02_00_00_00_00, 8);
    exp(&s, 0x582, 0x60_00_62_02_00_00_00_00, 8);

    // Set device to Operational
    send(&s, 0x000, 0x01_02, 2);

    // Send RPDO message twice.
    send(&s, 0x202, 0x0A_0B, 2);
    send(&s, 0x202, 0x0C_0D, 2);
    debug!("wait 1s");
    thread::sleep(Duration::from_millis(500));
    debug!("done");

    // Check the latest RPDO is written.
    send(&s, 0x602, 0x40_00_62_01_00_00_00_00, 8);
    exp(&s, 0x582, 0x4F_00_62_01_0C_00_00_00, 8);
    send(&s, 0x602, 0x40_00_62_02_00_00_00_00, 8);
    exp(&s, 0x582, 0x4F_00_62_02_0D_00_00_00, 8);

    send(&s, 0x000, 0x80_02, 2);

    Ok(())
}

pub fn rpdo_sync_mode(s: &CanSocket) -> Result<(), ErrorCode> {
    // Disable TPDOs
    send(&s, 0x602, 0x23_00_18_01_82_01_00_C0, 8);
    exp(&s, 0x582, 0x60_00_18_01_00_00_00_00, 8);
    send(&s, 0x602, 0x23_01_18_01_82_02_00_C0, 8);
    exp(&s, 0x582, 0x60_01_18_01_00_00_00_00, 8);

    // Set state to pre-operational and set 0x1800 related.
    send(&s, 0x000, 0x80_02, 2);
    // Write value C0000202h to object 1400h:01h
    send(&s, 0x602, 0x23_00_14_01_02_02_00_C0, 8);
    exp(&s, 0x582, 0x60_00_14_01_00_00_00_00, 8);
    // Write value 0Ah to object 1400h:02h
    send(&s, 0x602, 0x2F_00_14_02_0A_00_00_00, 8);
    exp(&s, 0x582, 0x60_00_14_02_00_00_00_00, 8);
    // Write value 0h to object 1400h:05h
    send(&s, 0x602, 0x2B_00_14_05_00_00_00_00, 8);
    exp(&s, 0x582, 0x60_00_14_05_00_00_00_00, 8);
    // Write value 40000202h to object 1400h:01h, enable the RPDO object
    send(&s, 0x602, 0x23_00_14_01_02_02_00_40, 8);
    exp(&s, 0x582, 0x60_00_14_01_00_00_00_00, 8);

    send(&s, 0x602, 0x40_00_16_00_00_00_00_00, 8);
    exp(&s, 0x582, 0x4F_00_16_00_02_00_00_00, 8);
    send(&s, 0x602, 0x40_00_16_01_00_00_00_00, 8);
    exp(&s, 0x582, 0x43_00_16_01_08_01_00_62, 8);
    send(&s, 0x602, 0x40_00_16_02_00_00_00_00, 8);
    exp(&s, 0x582, 0x43_00_16_02_08_02_00_62, 8);

    send(&s, 0x602, 0x2F_00_62_01_00_00_00_00, 8);
    exp(&s, 0x582, 0x60_00_62_01_00_00_00_00, 8);
    send(&s, 0x602, 0x2F_00_62_02_00_00_00_00, 8);
    exp(&s, 0x582, 0x60_00_62_02_00_00_00_00, 8);

    // Set device to Operational
    send(&s, 0x000, 0x01_02, 2);
    for _ in 0..3 {
        send(&s, 0x202, 0x0A_0B, 2);
        // Send 10 SYNC frame
        for _ in 0..10 { send(&s, 0x080, 0x0, 0); }
        // Expect [0x6200:01] = 0A, [0x6200:02] = 0B
        send(&s, 0x602, 0x40_00_62_01_00_00_00_00, 8);
        exp(&s, 0x582, 0x4F_00_62_01_0A_00_00_00, 8);
        send(&s, 0x602, 0x40_00_62_02_00_00_00_00, 8);
        exp(&s, 0x582, 0x4F_00_62_02_0B_00_00_00, 8);
    }
    send(&s, 0x000, 0x80_02, 2);

    Ok(())
}