use crate::prelude::*;

use core::str::FromStr;
use embedded_can::{Frame, Id, StandardId};
use embedded_can::Id::Standard;

pub trait ParseRadix: FromStr {
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::Err>
    where
        Self: Sized;
}

macro_rules! impl_parse_radix_signed {
    ($signed:ty, $unsigned:ty, $limit:expr, $upscale:ty, $wrap_around:expr) => {
        impl ParseRadix for $signed {
            fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::Err> {
                let val = <$unsigned>::from_str_radix(s, radix)?;
                if val <= $limit {
                    Ok(val as $signed)
                } else {
                    Ok((val as $upscale - $wrap_around) as $signed)
                }
            }
        }
    };
}

impl_parse_radix_signed!(i8, u8, 0x7F, i16, 0x100);
impl_parse_radix_signed!(i16, u16, 0x7FFF, i32, 0x10000);
impl_parse_radix_signed!(i32, u32, 0x7FFFFFFF, i64, 0x100000000);
impl_parse_radix_signed!(i64, u64, 0x7FFFFFFFFFFFFFFF, i128, 0x10000000000000000);

macro_rules! impl_parse_radix_for {
    ($t:ty) => {
        impl ParseRadix for $t {
            fn from_str_radix(s: &str, radix: u32) -> Result<Self, <Self as FromStr>::Err> {
                <$t>::from_str_radix(s, radix)
            }
        }
    };
}

impl_parse_radix_for!(u8);
impl_parse_radix_for!(u16);
impl_parse_radix_for!(u32);
impl_parse_radix_for!(u64);

pub fn parse_number<T: ParseRadix + Default>(s: &str) -> T {
    if s.starts_with("0x") || s.starts_with("0X") {
        T::from_str_radix(&s[2..], 16).unwrap_or_default()
    } else {
        s.parse().unwrap_or_default()
    }
}

pub fn result_to_option<T, Err>(res: Result<T, Err>) -> Option<T> {
    match res {
        Ok(val) => Some(val),
        Err(_) => None,
    }
}

pub fn to_value_with_node_id(node_id: u16, expression: &str) -> String {
    // Replace $NODEID with the actual node_id
    let modified_expression = expression.replace("$NODEID", &node_id.to_string());

    // Evaluate simple arithmetic expressions
    let value_sum: i64 = modified_expression
        .split('+')
        .filter_map(|s| {
            let s_trimmed = s.trim();
            if s_trimmed.starts_with("0x") || s_trimmed.starts_with("0X") {
                i64::from_str_radix(&s_trimmed[2..], 16).ok()
            } else {
                s_trimmed.parse::<i64>().ok()
            }
        })
        .sum();

    // Convert the evaluated sum to a string
    value_sum.to_string()
}

pub fn get_cob_id<F: Frame>(frame: &F) -> Option<u16> {
    if let Id::Standard(sid) = frame.id() {
        return Some(sid.as_raw());
    }
    None
}

fn is_hex_char(c: char) -> bool {
    ('0'..='9').contains(&c) || ('a'..='f').contains(&c) || ('A'..='F').contains(&c)
}

pub fn is_top(s: &str) -> bool {
    s.len() == 4 && s.chars().all(is_hex_char)
}

pub fn is_sub(s: &str) -> Option<(u16, u8)> {
    if s.len() > 7 && s[4..7].eq_ignore_ascii_case("sub") && s[0..4].chars().all(is_hex_char) {
        let (index_str, sub_str) = (&s[0..4], &s[7..]);
        match (u16::from_str_radix(index_str, 16), u8::from_str(sub_str)) {
            (Ok(index), Ok(sub)) => Some((index, sub)),
            _ => None,
        }
    } else {
        None
    }
}

pub fn is_name(s: &str) -> Option<u16> {
    s.ends_with("Name")
        .then(|| s[0..4].chars().all(is_hex_char))
        .and_then(|valid| valid.then(|| u16::from_str_radix(&s[0..4], 16).ok()))
        .flatten()
}

pub fn get_index_from_can_frame<F: Frame>(frame: &F) -> (u16, u8) {
    (
        u16::from_le_bytes([frame.data()[1], frame.data()[2]]),
        frame.data()[3],
    )
}

pub fn flatten(slices: &[&[u8]]) -> Vec<u8> {
    let mut res = Vec::new();
    for slice in slices {
        res.extend_from_slice(slice);
    }
    let l = res.len();
    if l > 8 {
        res.resize(8, 0);
    } else {
        for _ in l..8 {
            res.push(0);
        }
    }
    res
}

pub fn u64_to_vec(mut data: u64, bytes: usize) -> Vec<u8> {
    let mut res = vec![0u8; bytes];
    for i in 0..bytes {
        res[bytes - 1 - i] = (data & 0xFF) as u8;
        data = data >> 8;
    }
    res
}

pub fn genf_and_padding<F: Frame + Debug>(cob_id: u16, bytes: &[u8]) -> F {
    // Ensure bytes doesn't exceed 8 in length
    let cutoff = if bytes.len() > 8 { 8 } else { bytes.len() };
    let mut packet = bytes[0..cutoff].to_vec();

    // Padding to 8 bytes with 0
    while packet.len() < 8 {
        packet.push(0);
    }

    F::new(StandardId::new(cob_id).unwrap(), packet.as_slice()).unwrap()
}

pub fn genf<F: Frame + Debug>(cob_id: u16, bytes: &[u8]) -> F {
    F::new(StandardId::new(cob_id).unwrap(), bytes).unwrap()
}

static CCITT_HASH: [u16; 256] = [
    0x0000, 0x1021, 0x2042, 0x3063, 0x4084, 0x50a5, 0x60c6, 0x70e7, 0x8108, 0x9129, 0xa14a, 0xb16b,
    0xc18c, 0xd1ad, 0xe1ce, 0xf1ef, 0x1231, 0x0210, 0x3273, 0x2252, 0x52b5, 0x4294, 0x72f7, 0x62d6,
    0x9339, 0x8318, 0xb37b, 0xa35a, 0xd3bd, 0xc39c, 0xf3ff, 0xe3de, 0x2462, 0x3443, 0x0420, 0x1401,
    0x64e6, 0x74c7, 0x44a4, 0x5485, 0xa56a, 0xb54b, 0x8528, 0x9509, 0xe5ee, 0xf5cf, 0xc5ac, 0xd58d,
    0x3653, 0x2672, 0x1611, 0x0630, 0x76d7, 0x66f6, 0x5695, 0x46b4, 0xb75b, 0xa77a, 0x9719, 0x8738,
    0xf7df, 0xe7fe, 0xd79d, 0xc7bc, 0x48c4, 0x58e5, 0x6886, 0x78a7, 0x0840, 0x1861, 0x2802, 0x3823,
    0xc9cc, 0xd9ed, 0xe98e, 0xf9af, 0x8948, 0x9969, 0xa90a, 0xb92b, 0x5af5, 0x4ad4, 0x7ab7, 0x6a96,
    0x1a71, 0x0a50, 0x3a33, 0x2a12, 0xdbfd, 0xcbdc, 0xfbbf, 0xeb9e, 0x9b79, 0x8b58, 0xbb3b, 0xab1a,
    0x6ca6, 0x7c87, 0x4ce4, 0x5cc5, 0x2c22, 0x3c03, 0x0c60, 0x1c41, 0xedae, 0xfd8f, 0xcdec, 0xddcd,
    0xad2a, 0xbd0b, 0x8d68, 0x9d49, 0x7e97, 0x6eb6, 0x5ed5, 0x4ef4, 0x3e13, 0x2e32, 0x1e51, 0x0e70,
    0xff9f, 0xefbe, 0xdfdd, 0xcffc, 0xbf1b, 0xaf3a, 0x9f59, 0x8f78, 0x9188, 0x81a9, 0xb1ca, 0xa1eb,
    0xd10c, 0xc12d, 0xf14e, 0xe16f, 0x1080, 0x00a1, 0x30c2, 0x20e3, 0x5004, 0x4025, 0x7046, 0x6067,
    0x83b9, 0x9398, 0xa3fb, 0xb3da, 0xc33d, 0xd31c, 0xe37f, 0xf35e, 0x02b1, 0x1290, 0x22f3, 0x32d2,
    0x4235, 0x5214, 0x6277, 0x7256, 0xb5ea, 0xa5cb, 0x95a8, 0x8589, 0xf56e, 0xe54f, 0xd52c, 0xc50d,
    0x34e2, 0x24c3, 0x14a0, 0x0481, 0x7466, 0x6447, 0x5424, 0x4405, 0xa7db, 0xb7fa, 0x8799, 0x97b8,
    0xe75f, 0xf77e, 0xc71d, 0xd73c, 0x26d3, 0x36f2, 0x0691, 0x16b0, 0x6657, 0x7676, 0x4615, 0x5634,
    0xd94c, 0xc96d, 0xf90e, 0xe92f, 0x99c8, 0x89e9, 0xb98a, 0xa9ab, 0x5844, 0x4865, 0x7806, 0x6827,
    0x18c0, 0x08e1, 0x3882, 0x28a3, 0xcb7d, 0xdb5c, 0xeb3f, 0xfb1e, 0x8bf9, 0x9bd8, 0xabbb, 0xbb9a,
    0x4a75, 0x5a54, 0x6a37, 0x7a16, 0x0af1, 0x1ad0, 0x2ab3, 0x3a92, 0xfd2e, 0xed0f, 0xdd6c, 0xcd4d,
    0xbdaa, 0xad8b, 0x9de8, 0x8dc9, 0x7c26, 0x6c07, 0x5c64, 0x4c45, 0x3ca2, 0x2c83, 0x1ce0, 0x0cc1,
    0xef1f, 0xff3e, 0xcf5d, 0xdf7c, 0xaf9b, 0xbfba, 0x8fd9, 0x9ff8, 0x6e17, 0x7e36, 0x4e55, 0x5e74,
    0x2e93, 0x3eb2, 0x0ed1, 0x1ef0,
];

pub fn crc16_canopen_with_lut(bytes: &[u8]) -> u16 {
    let mut crc: u16 = 0x0000;

    for byte in bytes {
        let table_idx = ((crc >> 8) ^ (*byte as u16)) as usize;
        crc = CCITT_HASH[table_idx] ^ (crc << 8);
    }

    crc
}

#[cfg(test)]
mod util_tests {
    use super::to_value_with_node_id;
    use crate::util::parse_number;

    #[test]
    fn test_to_value_with_node_id() {
        assert_eq!(to_value_with_node_id(2, "$NODEID + 100"), "102");
        assert_eq!(to_value_with_node_id(2, "100+$NODEID"), "102");
        assert_eq!(to_value_with_node_id(2, "100"), "100");
        assert_eq!(to_value_with_node_id(2, "$NODEID+100+200"), "302");
        assert_eq!(to_value_with_node_id(2, "$NODEID + 100 + 200"), "302");
        assert_eq!(to_value_with_node_id(1234, "$NODEID + 100 + 200"), "1534");
        assert_eq!(to_value_with_node_id(2, "No arithmetic here"), "0");
        assert_eq!(to_value_with_node_id(2, "$NODEID+0x600"), "1538");
    }

    #[test]
    fn test_parse_number_i8() {
        assert_eq!(parse_number::<i8>("0xFF"), -1);
        assert_eq!(parse_number::<i8>("0x7F"), 127);
        assert_eq!(parse_number::<i8>("-128"), -128);
        assert_eq!(parse_number::<i8>("0"), 0);
        assert_eq!(parse_number::<i8>("0xAB"), -85); // 0xAB in two's complement for i8 is -85
        assert_eq!(parse_number::<i8>("abc"), 0); // Invalid input returns default
    }

    #[test]
    fn test_parse_number_u8() {
        assert_eq!(parse_number::<u8>("0xFF"), 255);
        assert_eq!(parse_number::<u8>("0"), 0);
        assert_eq!(parse_number::<u8>("255"), 255);
        assert_eq!(parse_number::<u8>("abc"), 0); // Invalid input returns default
    }

    #[test]
    fn test_parse_number_i32() {
        assert_eq!(parse_number::<i32>("0x7FFFFFFF"), 2_147_483_647);
        assert_eq!(parse_number::<i32>("-2147483648"), -2_147_483_648);
        assert_eq!(parse_number::<i32>("0"), 0);
        assert_eq!(parse_number::<i32>("abc"), 0); // Invalid input returns default
    }

    #[test]
    fn test_parse_number_u32() {
        assert_eq!(parse_number::<u32>("0xFFFFFFFF"), 4_294_967_295);
        assert_eq!(parse_number::<u32>("0"), 0);
        assert_eq!(parse_number::<u32>("4294967295"), 4_294_967_295);
        assert_eq!(parse_number::<u32>("abc"), 0); // Invalid input returns default
    }

    #[test]
    fn test_crc16_ccitt() {
        let s = "CANopenDemoPIC32";
        let crc = crate::util::crc16_canopen_with_lut(s.as_bytes());
        assert_eq!(crc, 0x43F3, "({:x} != 0x43F3)", crc);
    }
}
