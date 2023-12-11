use socketcan::CanSocket;
use crate::util::{exp, send};

pub fn heartbeat_basic(s: &CanSocket) {
    send(&s, 0x000, 0x81_02, 2);
    send(&s, 0x602, 0x2B_17_10_00_0A_00_00_00, 8);
    exp(&s, 0x582, 0x60_17_10_00_00_00_00_00, 8);
    for _ in 0..3 {
        exp(&s, 0x702, 0x0, 1);
    }
    send(&s, 0x602, 0x2B_17_10_00_00_00_00_00, 8);
    exp(&s, 0x582, 0x60_17_10_00_00_00_00_00, 8);
}