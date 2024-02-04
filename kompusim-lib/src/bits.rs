pub trait BitOps {
    // return true if bit self[idx] is 1
    fn bit(self, idx: u32) -> bool;
    // return bits self[end:start] shifted to LSB
    fn bits(self, end: u32, start: u32) -> Self;
    // xor bits self[end:start]
    fn xor(self, end: u32, start: u32) -> Self;
    // Clean bits [end:start]
    fn rst_bits(self, end: u32, start: u32) -> Self;
}

impl BitOps for u8 {
    // TODO: move to trait?
    #[inline(always)]
    fn bit(self, idx: u32) -> bool {
        assert!(idx < Self::BITS);
        self & (1 << idx) != 0
    }

    fn bits(self, end: u32, start: u32) -> Self {
        assert!(end < Self::BITS && start < Self::BITS && end >= start);
        (self >> start) & (Self::MAX >> (7 - (end - start)))
    }

    fn xor(self, end: u32, start: u32) -> Self {
        assert!(end < Self::BITS && start < Self::BITS && end >= start);
        let m = Self::MAX >> start << start << (Self::BITS - end - 1) >> (Self::BITS - end - 1);
        self ^ m
    }

    // Clean bits [end:start]
    fn rst_bits(self, end: u32, start: u32) -> Self {
        let mask = Self::MAX.xor(end, start);
        self & mask
    }
}

impl BitOps for u16 {
    // TODO: move to trat?
    #[inline(always)]
    fn bit(self, idx: u32) -> bool {
        assert!(idx < Self::BITS);
        self & (1 << idx) != 0
    }

    fn bits(self, end: u32, start: u32) -> Self {
        assert!(end < Self::BITS && start < Self::BITS && end >= start);
        (self >> start) & (Self::MAX >> (15 - (end - start)))
    }

    fn xor(self, end: u32, start: u32) -> Self {
        assert!(end < Self::BITS && start < Self::BITS && end >= start);
        let m = Self::MAX >> start << start << (Self::BITS - end - 1) >> (Self::BITS - end - 1);
        self ^ m
    }

    // Clean bits [end:start]
    fn rst_bits(self, end: u32, start: u32) -> Self {
        let mask = Self::MAX.xor(end, start);
        self & mask
    }
}

impl BitOps for i16 {
    #[inline(always)]
    fn bit(self, idx: u32) -> bool {
        assert!(idx < Self::BITS);
        self as u16 & (1_u16 << idx) != 0
    }

    fn bits(self, end: u32, start: u32) -> Self {
        assert!(end < Self::BITS && start < Self::BITS && end >= start);
        ((self as u16 >> start) & (u16::MAX >> (u16::BITS - 1 - (end - start)))) as i16
    }

    fn xor(self, end: u32, start: u32) -> Self {
        assert!(end < Self::BITS && start < Self::BITS && end >= start);
        let m: u16 = u16::MAX >> start << start << (u16::BITS - end - 1) >> (u16::BITS - end - 1);
        (self as u16 ^ m) as i16
    }

    // Clean bits [end:start]
    fn rst_bits(self, end: u32, start: u32) -> Self {
        let mask = Self::MAX.xor(end, start);
        self & mask
    }
}

impl BitOps for u32 {
    #[inline(always)]
    fn bit(self, idx: u32) -> bool {
        assert!(idx < Self::BITS);
        self & (1 << idx) != 0
    }

    fn bits(self, end: u32, start: u32) -> Self {
        assert!(end < Self::BITS && start < Self::BITS && end >= start);
        (self >> start) & (Self::MAX >> (Self::BITS - 1 - (end - start)))
    }

    fn xor(self, end: u32, start: u32) -> Self {
        assert!(end < Self::BITS && start < Self::BITS && end >= start);
        let m = Self::MAX >> start << start << (Self::BITS - end - 1) >> (Self::BITS - end - 1);
        self ^ m
    }

    // Clean bits [end:start]
    fn rst_bits(self, end: u32, start: u32) -> Self {
        let mask = Self::MAX.xor(end, start);
        self & mask
    }
}

impl BitOps for u64 {
    #[inline(always)]
    fn bit(self, idx: u32) -> bool {
        assert!(idx < Self::BITS);
        self & (1 << idx) != 0
    }

    fn bits(self, end: u32, start: u32) -> Self {
        assert!(end < Self::BITS && start < Self::BITS && end >= start);
        (self >> start) & (Self::MAX >> (Self::BITS - 1 - (end - start)))
    }

    // XOR bits [end:start]
    fn xor(self, end: u32, start: u32) -> Self {
        assert!(end < Self::BITS && start < Self::BITS && end >= start);
        let m = Self::MAX >> start << start << (Self::BITS - end - 1) >> (Self::BITS - end - 1);
        self ^ m
    }

    // Clean bits [end:start]
    fn rst_bits(self, end: u32, start: u32) -> Self {
        let mask = Self::MAX.xor(end, start);
        self & mask
    }
}

#[test]
fn test_bit_ops() {
    // test keyword 'as'
    assert!(-1_i32 as u32 == u32::MAX);

    // u8
    assert!(0b_0000_1100_u8.bits(3, 2) == 0x3_u8);
    assert!(0b_0000_1100_u8.bits(7, 0) == 0xc_u8);
    assert!(0b_1111_1111_u8.bits(7, 7) == 0x1_u8);

    // .bit()
    assert!(0b_0001_u16.bit(0));
    assert!(!0b_0001_u16.bit(1));
    assert!(!0b_0001_u16.bit(15));

    // u16
    assert!(0b_00000000_00000111_u16.bits(2, 0) == 0b_111_u16);
    assert!(0b_00000000_00000110_u16.bits(2, 1) == 0b_11_u16);
    assert!(0b_10000000_00000000_u16.bits(15, 15) == 1_u16);
    assert!(0xffff_u16.bits(15, 0) == 0xffff_u16);
    assert!(0b_0110_u16.bits(2, 1) != 0b_01_u16);
    assert!(0xffff_u16.bits(15, 0) != 0xfffe_u16);
    // xor
    assert!(0xffff_u16.xor(11, 8) == 0xf0ff);
    assert!(0xffff_u16.xor(3, 0) == 0xfff0);
    assert!(0x0000_u16.xor(7, 7) == 0x0080);
    assert!(0x0000_u16.xor(15, 15) == 0x8000);
    assert!(0xaaaa_u16.xor(15, 0) == 0x5555);

    // i16
    assert_eq!(1_i16.bits(0, 0), 1);
    assert_eq!(0xa5_i16.bits(7, 4), 0xa);
    assert_eq!(0x33_i16.bits(1, 1), 1);
    assert_eq!(0x33_i16.bits(5, 5), 1);
    assert_eq!((-1_i16).bits(15, 0), -1);

    // u32
    assert!(0xffff_ff0f_u32.bits(11, 0) == 0xf0f_u32);
    assert!(0x8fff_0f0f_u32.bits(31, 31) == 1_u32);

    assert!(0b0010_1001_1000_0110_0011.bits(14, 12) == 0b_001_u32);
    //              ^^^
    assert!(0xffff_ffff_u32.xor(11, 8) == 0xffff_f0ff_u32);
    assert!(0xffff_ffff_u32.xor(3, 0) == 0xffff_fff0_u32);
    assert!(0x0000_8000_u32.xor(15, 15) == 0);
    assert!(0xaaaa_5555_u32.xor(15, 0) == 0xaaaa_aaaa_u32);
    assert!(0xaaaa_5555_u32.xor(31, 0) == 0x5555_aaaa_u32);
    assert!(0xffff_ffff_u32.xor(0, 0) == 0xffff_fffe_u32);

    // u64
    assert!(u64::MAX.bits(11, 0) == 0xfff_u64);
    assert!(u64::MAX.xor(0, 0) == 0xffff_ffff_ffff_fffe);
    assert!(u64::MAX.rst_bits(0, 0) == 0xffff_ffff_ffff_fffe);
}
