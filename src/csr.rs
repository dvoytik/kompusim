const MHARTID: u16 = 0xf14;

pub fn csr_r64(csr_a: u16) -> u64 {
    match csr_a {
        MHARTID => 0, // current cpu id
        _ => panic!("not implemented"),
    }
}

pub fn csr_w64(csr_a: u16, val: u64) {
    match csr_a {
        MHARTID => (), // ignore
        _ => panic!("not implemented"),
    }
}
