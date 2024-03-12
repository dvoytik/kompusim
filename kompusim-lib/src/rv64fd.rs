// RISC-V "F" and "D" extension - single and double precision floating-point

#[derive(Clone, Debug, Default)]
pub struct RV64FDRegs {
    // f0 - f31 registers
    pub f: [u64; 32],
    /// floating-point control and status register
    pub fcsr: u64,
}
