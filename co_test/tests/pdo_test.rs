use std::panic;
use log::{error, info};
use socketcan::Socket;

use canopen::util::{genf, genf_and_padding, u64_to_vec};
use co_test::async_util::AsyncExpector;
use co_test::util::{exp, INTERFACE_NAME, send, sendf};

use crate::testing::CONTEXT;

mod testing;

#[test]
fn test_tpdo_for_event_driven_mode() {
    // TODO(zephyr): use only one Expector, including sync / async expect.
    let _ = panic::take_hook();
    panic::set_hook(Box::new(|panic_info| {
        error!("{}", panic_info);
        std::process::exit(1);
    }));

    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(INTERFACE_NAME).expect("Failed to open CAN socket");

    // Set state to pre-operational and set 0x1800 related.
    sendf(&s, 0x000, 0x80_02, 2);
    // Write value C0000184h to object 1800h:01h
    send(&s, &genf_and_padding(0x602, &u64_to_vec(0x23_00_18_01_82_01_00_C0, 8)));
    exp(&s, &genf_and_padding(0x582, &u64_to_vec(0x60_00_18_01_00_00_00_00, 8)));
    // Write value FEh to object 1800h:02h
    send(&s, &genf_and_padding(0x602, &u64_to_vec(0x2F_00_18_02_FE_00_00_00, 8)));
    exp(&s, &genf_and_padding(0x582, &u64_to_vec(0x60_00_18_02_00_00_00_00, 8)));
    // Write value 64h to object 1800h:05h
    send(&s, &genf_and_padding(0x602, &u64_to_vec(0x2B_00_18_05_64_00_00_00, 8)));
    exp(&s, &genf_and_padding(0x582, &u64_to_vec(0x60_00_18_05_00_00_00_00, 8)));
    // Write value 40000184h to object 1800h:01h
    send(&s, &genf_and_padding(0x602, &u64_to_vec(0x23_00_18_01_82_01_00_40, 8)));
    exp(&s, &genf_and_padding(0x582, &u64_to_vec(0x60_00_18_01_00_00_00_00, 8)));

    {
        // Need to scope the ExpectContainer.
        let mut ec = AsyncExpector::new();
        // Set device to Operational
        sendf(&s, 0x000, 0x01_02, 2);
        for _ in 0..10 {
            ec.async_expect(genf(0x182, &u64_to_vec(0xFF_00, 2)));
        }
        ec.async_expect(genf(0x282, &u64_to_vec(0xFF_03_FF_03_FF_03_FF_03, 8)));
        let res = ec.wait_for_all();
        info!("xfguo: result of wait_for_all(): res = '{}', len = {}", res, res.len());
        assert_eq!(res.as_str(), "");
        sendf(&s, 0x000, 0x80_02, 2);
        info!("xfguo: done");
    }
}
