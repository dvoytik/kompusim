pub trait Bit {
    fn bit(self, idx: usize) -> bool;
    fn bits(self, end: usize, start: usize) -> Self;
}

impl Bit for u16 {
    #[inline(always)]
    fn bit(self, idx: usize) -> bool {
        assert!(idx <= 15, "index out of bound");
        if self & (1_u16 << idx) == 0 {
            false
        } else {
            true
        }
    }

    fn bits(self, end: usize, start: usize) -> Self {
        assert!(
            end <= 15 && start <= 15 && end >= start,
            "wrong start and end"
        );
        (self >> start) & (0xffff >> (15 - (end - start)))
    }
}

impl Bit for u32 {
    #[inline(always)]
    fn bit(self, idx: usize) -> bool {
        assert!(idx <= 31, "index out of bound");
        if self & (1_u32 << idx) == 0 {
            false
        } else {
            true
        }
    }

    fn bits(self, end: usize, start: usize) -> Self {
        assert!(
            end <= 31 && start <= 31 && end >= start,
            "wrong start and end"
        );
        (self >> start) & (0xffff_ffff >> (31 - (end - start)))
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
    // u32
    assert!(0xffff_ff0f_u32.bits(11, 0) == 0xf0f_u32);
    assert!(0x8fff_0f0f_u32.bits(31, 31) == 1_u32);

    println!("{:x}", 0xfff_ffff >> (31 - 14));
    println!("{:x}", 0xfff_ffff >> (14));
    assert!(0b0010_1001_1000_0110_0011.bits(14, 12) == 0b_001_u32);
    //              ^^^
}
