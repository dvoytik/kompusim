pub trait Bit {
    // return true if bit self[idx] is 1
    fn bit(self, idx: u32) -> bool;
    // return bits self[end:start] shifted to LSB
    fn bits(self, end: u32, start: u32) -> Self;
    // xor bits self[end:start]
    fn xor(self, end: u32, start: u32) -> Self;
}

impl Bit for u16 {
    #[inline(always)]
    fn bit(self, idx: u32) -> bool {
        assert!(idx < Self::BITS);
        if self & (1_u16 << idx) == 0 {
            false
        } else {
            true
        }
    }

    fn bits(self, end: u32, start: u32) -> Self {
        assert!(end < Self::BITS && start < Self::BITS && end >= start);
        (self >> start) & (0xffff >> (15 - (end - start)))
    }

    fn xor(self, end: u32, start: u32) -> Self {
        assert!(end < Self::BITS && start < Self::BITS && end >= start);
        let m = Self::MAX >> start << start << (Self::BITS - end - 1) >> (Self::BITS - end - 1);
        self ^ m
    }
}

impl Bit for u32 {
    #[inline(always)]
    fn bit(self, idx: u32) -> bool {
        assert!(idx < Self::BITS);
        if self & (1_u32 << idx) == 0 {
            false
        } else {
            true
        }
    }

    fn bits(self, end: u32, start: u32) -> Self {
        assert!(end < Self::BITS && start < Self::BITS && end >= start);
        (self >> start) & (0xffff_ffff >> (31 - (end - start)))
    }
    fn xor(self, end: u32, start: u32) -> Self {
        assert!(end < Self::BITS && start < Self::BITS && end >= start);
        let m = Self::MAX >> start << start << (Self::BITS - end - 1) >> (Self::BITS - end - 1);
        self ^ m
    }
}

#[test]
fn test_u16_bit() {
    assert!(0b_0001_u16.bit(0) == true);
    assert!(0b_0001_u16.bit(1) == false);
    assert!(0b_0001_u16.bit(15) == false);

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
}
