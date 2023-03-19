use crate::bits::BitOps;

pub struct I12(pub i16);

pub trait Imm {
    fn add_i12(self, v: I12) -> Self;
    fn add_i21(self, v: I21) -> Self;
}

impl Imm for u64 {
    // Overflow is ignored
    fn add_i12(self, v: I12) -> u64 {
        (self as i64).wrapping_add(v.0 as i64) as u64
    }
    // Overflow is ignored
    fn add_i21(self, v: I21) -> u64 {
        (self as i64).wrapping_add(v.0 as i64) as u64
    }
}

impl I12 {
    pub fn from_u16(v: u16) -> Self {
        if v.bit(11) {
            I12((u16::MAX.xor(10, 0) | v.bits(10, 0)) as i16)
        } else {
            I12(v as i16)
        }
    }

    #[allow(dead_code)]
    pub fn from_i16(v: i16) -> Self {
        I12(v)
    }
}

pub struct I21(pub i32);

impl I21 {
    pub fn from_u32(v: u32) -> Self {
        if v.bit(20) {
            I21((u32::MAX.xor(19, 0) | v.bits(19, 0)) as i32)
        } else {
            I21(v as i32)
        }
    }

    #[allow(dead_code)]
    pub fn from_i32(v: i32) -> Self {
        I21(v)
    }
}

#[test]
fn test_imm12() {
    assert!((u64::MAX).add_i12(I12::from_i16(0)) == u64::MAX);
    // overflow
    assert!((u64::MAX).add_i12(I12::from_i16(1)) == u64::MIN);

    assert!((u64::MAX).add_i12(I12::from_i16(-1)) == u64::MAX - 1);
    assert!((u64::MAX).add_i12(I12::from_u16(0xfff)) == u64::MAX - 1);

    assert!(5000_u64.add_i12(I12::from_u16(0x800)) == 5000 - 2048);
    assert!(5000_u64.add_i12(I12::from_i16(-2048)) == 5000 - 2048);

    assert!(5000_000_u64.add_i12(I12::from_u16(0x7ff)) == 5000_000 + 2047);
}

#[test]
fn test_imm21() {
    assert!((u64::MAX).add_i21(I21::from_i32(0)) == u64::MAX);
    // overflow
    assert!((u64::MAX).add_i21(I21::from_i32(1)) == u64::MIN);

    assert!((u64::MAX).add_i21(I21::from_i32(-1)) == u64::MAX - 1);
    assert!((u64::MAX).add_i21(I21::from_u32(0x1f_ffff)) == u64::MAX - 1);

    assert!(5000_000_u64.add_i21(I21::from_u32(0x10_0000)) == 5000_000 - 1024 * 1024);
    assert!(5000_000_u64.add_i21(I21::from_i32(-1024 * 1024)) == 5000_000 - 1024 * 1024);

    assert!(5000_000_u64.add_i21(I21::from_u32(0x7ff)) == 5000_000 + 2047);
}
