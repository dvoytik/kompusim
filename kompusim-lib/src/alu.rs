use core::fmt;
use std::fmt::{Display, LowerHex};

use crate::bits::BitOps;

pub trait Imm {
    fn add_i9(self, v: I9) -> Self;
    fn add_i12(self, v: I12) -> Self;
    fn add_i13(self, v: I13) -> Self;
    fn add_i21(self, v: I21) -> Self;
}

impl Imm for u64 {
    // Overflow is ignored
    fn add_i9(self, v: I9) -> u64 {
        (self as i64).wrapping_add(v.0 as i64) as u64
    }
    // Overflow is ignored
    fn add_i12(self, v: I12) -> u64 {
        (self as i64).wrapping_add(v.0 as i64) as u64
    }
    // Overflow is ignored
    fn add_i13(self, v: I13) -> u64 {
        (self as i64).wrapping_add(v.0 as i64) as u64
    }
    // Overflow is ignored
    fn add_i21(self, v: I21) -> u64 {
        (self as i64).wrapping_add(v.0 as i64) as u64
    }
}

impl Imm for u32 {
    // Overflow is ignored
    fn add_i9(self, v: I9) -> u32 {
        (self as i32).wrapping_add(v.0 as i32) as u32
    }
    // Overflow is ignored
    fn add_i12(self, v: I12) -> u32 {
        (self as i32).wrapping_add(v.0 as i32) as u32
    }
    // Overflow is ignored
    fn add_i13(self, v: I13) -> u32 {
        (self as i32).wrapping_add(v.0 as i32) as u32
    }
    // Overflow is ignored
    fn add_i21(self, v: I21) -> u32 {
        (self as i32).wrapping_add(v.0) as u32
    }
}

/// Immidiate signed 6 bit
#[derive(Copy, Clone)]
pub struct I6(pub i8);

impl From<u16> for I6 {
    fn from(v: u16) -> I6 {
        if v.bit(5) {
            I6((u16::MAX.xor(4, 0) | v.bits(4, 0)) as i8)
        } else {
            I6(v as i8)
        }
    }
}

impl From<I6> for i8 {
    fn from(v: I6) -> i8 {
        v.0
    }
}

impl From<I6> for u64 {
    fn from(v: I6) -> u64 {
        v.0 as u64
    }
}

impl Display for I6 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Used for format!()
impl LowerHex for I6 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

/// Immidiate signed 9 bit
#[derive(Copy, Clone)]
pub struct I9(pub i16);

impl From<u16> for I9 {
    fn from(v: u16) -> I9 {
        if v.bit(8) {
            I9((u16::MAX.xor(7, 0) | v.bits(7, 0)) as i16)
        } else {
            I9(v as i16)
        }
    }
}

/// Immidiate signed 12 bit
#[derive(Copy, Clone)]
pub struct I12(pub i16);

impl From<u16> for I12 {
    fn from(v: u16) -> I12 {
        if v.bit(11) {
            I12((u16::MAX.xor(10, 0) | v.bits(10, 0)) as i16)
        } else {
            I12(v as i16)
        }
    }
}

impl From<I12> for u64 {
    fn from(v: I12) -> u64 {
        v.0 as i64 as u64
    }
}

impl From<i16> for I12 {
    fn from(v: i16) -> I12 {
        I12(v)
    }
}

impl From<I6> for I12 {
    fn from(v: I6) -> I12 {
        I12(v.0 as i16)
    }
}

impl From<u8> for I12 {
    fn from(v: u8) -> I12 {
        I12(v as i16)
    }
}

impl Display for I12 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Used for format!()
impl LowerHex for I12 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

/// Immidiate signed 13 bit
#[derive(Copy, Clone)]
pub struct I13(pub i16);

impl From<u16> for I13 {
    fn from(v: u16) -> I13 {
        if v.bit(12) {
            I13((u16::MAX.xor(11, 0) | v.bits(11, 0)) as i16)
        } else {
            I13(v as i16)
        }
    }
}

impl From<i16> for I13 {
    fn from(v: i16) -> I13 {
        I13(v)
    }
}

impl From<I9> for I13 {
    fn from(v: I9) -> I13 {
        I13(v.0)
    }
}

impl From<I13> for u64 {
    fn from(v: I13) -> u64 {
        v.0 as u64
    }
}

/// Used for format!()
impl LowerHex for I13 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

/// Immidiate signed 21 bit
#[derive(Copy, Clone)]
pub struct I21(pub i32);

impl From<u32> for I21 {
    fn from(v: u32) -> I21 {
        if v.bit(20) {
            I21((u32::MAX.xor(19, 0) | v.bits(19, 0)) as i32)
        } else {
            I21(v as i32)
        }
    }
}

impl From<i32> for I21 {
    fn from(v: i32) -> I21 {
        I21(v)
    }
}

impl From<I12> for I21 {
    fn from(v: I12) -> I21 {
        I21(v.0 as i32)
    }
}

impl From<I21> for u64 {
    fn from(v: I21) -> u64 {
        v.0 as u64
    }
}

/// Used for format!()
impl LowerHex for I21 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

/// Used for format!()
impl Display for I21 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[test]
fn test_rust_type_conversion() {
    // u64 to u32 conversion
    let u64_val = 0x_1234_5678_90ab_cdef_u64;
    assert_eq!(u64_val as u32, 0x_90ab_cdef_u32);
}

#[test]
fn test_imm12() {
    assert!((u64::MAX).add_i12(I12::from(0_i16)) == u64::MAX);
    // overflow
    assert!((u64::MAX).add_i12(I12::from(1_i16)) == u64::MIN);

    assert!((u64::MAX).add_i12(I12::from(-1_i16)) == u64::MAX - 1);
    assert!((u64::MAX).add_i12(I12::from(0xfff_u16)) == u64::MAX - 1);

    assert!(5000_u64.add_i12(I12::from(0x800_u16)) == 5000 - 2048);
    assert!(5000_u64.add_i12(I12::from(-2048_i16)) == 5000 - 2048);

    assert!(500_0000_u64.add_i12(I12::from(0x7ff_u16)) == 500_0000 + 2047);

    // bits
    assert_eq!(I12::from(1_u16).0.bits(0, 0), 1);
    assert_eq!(I12::from(0x_33_u16).0.bits(0, 0), 1);
    assert_eq!(I12::from(0x_33_u16).0.bits(1, 1), 1);
}

#[test]
fn test_imm13() {
    assert!((u64::MAX).add_i13(I13::from(0_i16)) == u64::MAX);
    // overflow
    assert!((u64::MAX).add_i13(I13::from(1_i16)) == u64::MIN);

    assert!((u64::MAX).add_i13(I13::from(-1_i16)) == u64::MAX - 1);
    assert!((u64::MAX).add_i13(I13::from(0x1fff_u16)) == u64::MAX - 1);

    assert!(5000_u64.add_i13(I13::from(0x1000_u16)) == 5000 - 4096);
    assert!(5000_u64.add_i13(I13::from(-4096_i16)) == 5000 - 4096);

    assert!(500_0000_u64.add_i13(I13::from(0xfff_u16)) == 500_0000 + 4095);
}

#[test]
fn test_imm21() {
    assert!((u64::MAX).add_i21(I21::from(0_i32)) == u64::MAX);
    // overflow
    assert!((u64::MAX).add_i21(I21::from(1_i32)) == u64::MIN);

    assert!((u64::MAX).add_i21(I21::from(-1_i32)) == u64::MAX - 1);
    assert!((u64::MAX).add_i21(I21::from(0x1f_ffff_u32)) == u64::MAX - 1);

    assert!(500_0000_u64.add_i21(I21::from(0x10_0000_u32)) == 500_0000 - 1024 * 1024);
    assert!(500_0000_u64.add_i21(I21::from(-1024_i32 * 1024)) == 500_0000 - 1024 * 1024);

    assert!(500_0000_u64.add_i21(I21::from(0x7ff)) == 500_0000 + 2047);

    // convert I12 to I21
    let imm21: I21 = I12(-22).into();
    assert!(imm21.0 == -22);
}

#[test]
fn test_imm6() {
    let a = u16::MAX;
    let b: I6 = a.into();
    let c: i8 = b.into();
    assert_eq!(c, -1);

    assert_eq!(u64::from(I6::from(0x_3f)), 0x_ffff_ffff_ffff_ffff_u64);
    assert_eq!(u64::from(I6::from(0x_20)), 0x_ffff_ffff_ffff_ffe0_u64);
}

#[test]
fn test_imm9() {
    assert_eq!(I9::from(u16::MAX).0, -1);
    assert_eq!(I9::from(0x1ff).0, -1);
    assert_eq!(I9::from(0xff).0, 255);
    assert_eq!(I9::from(0).0, 0);
}
