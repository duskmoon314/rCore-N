#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    _reserved_0_dll: [u8; 0x04],
    _reserved_1_dlh: [u8; 0x04],
    _reserved_2_fcr: [u8; 0x04],
    #[doc = "0x0c - UART Line Control Register"]
    pub lcr: LCR,
    #[doc = "0x10 - UART Modem Control Register"]
    pub mcr: MCR,
    #[doc = "0x14 - UART Line Status Register"]
    pub lsr: LSR,
    #[doc = "0x18 - UART Modem Status Register"]
    pub msr: MSR,
    #[doc = "0x1c - UART Scratch Register"]
    pub sch: SCH,
}
impl RegisterBlock {
    #[doc = "0x00 - UART Divisor Latch Low Register"]
    #[inline(always)]
    pub const fn dll(&self) -> &DLL {
        unsafe { &*(self as *const Self).cast::<u8>().add(0usize).cast() }
    }
    #[doc = "0x00 - UART Transmit Holding Register"]
    #[inline(always)]
    pub const fn thr(&self) -> &THR {
        unsafe { &*(self as *const Self).cast::<u8>().add(0usize).cast() }
    }
    #[doc = "0x00 - UART Receive Buffer Register"]
    #[inline(always)]
    pub const fn rbr(&self) -> &RBR {
        unsafe { &*(self as *const Self).cast::<u8>().add(0usize).cast() }
    }
    #[doc = "0x04 - UART Interrupt Enable Register"]
    #[inline(always)]
    pub const fn ier(&self) -> &IER {
        unsafe { &*(self as *const Self).cast::<u8>().add(4usize).cast() }
    }
    #[doc = "0x04 - UART Divisor Latch High Register"]
    #[inline(always)]
    pub const fn dlh(&self) -> &DLH {
        unsafe { &*(self as *const Self).cast::<u8>().add(4usize).cast() }
    }
    #[doc = "0x08 - UART FIFO Control Register"]
    #[inline(always)]
    pub const fn fcr(&self) -> &FCR {
        unsafe { &*(self as *const Self).cast::<u8>().add(8usize).cast() }
    }
    #[doc = "0x08 - UART Interrupt Identity Register"]
    #[inline(always)]
    pub const fn iir(&self) -> &IIR {
        unsafe { &*(self as *const Self).cast::<u8>().add(8usize).cast() }
    }
}
#[doc = "rbr (r) register accessor: an alias for `Reg<RBR_SPEC>`"]
pub type RBR = crate::Reg<rbr::RBR_SPEC>;
#[doc = "UART Receive Buffer Register"]
pub mod rbr;
#[doc = "thr (w) register accessor: an alias for `Reg<THR_SPEC>`"]
pub type THR = crate::Reg<thr::THR_SPEC>;
#[doc = "UART Transmit Holding Register"]
pub mod thr;
#[doc = "dll (rw) register accessor: an alias for `Reg<DLL_SPEC>`"]
pub type DLL = crate::Reg<dll::DLL_SPEC>;
#[doc = "UART Divisor Latch Low Register"]
pub mod dll;
#[doc = "dlh (rw) register accessor: an alias for `Reg<DLH_SPEC>`"]
pub type DLH = crate::Reg<dlh::DLH_SPEC>;
#[doc = "UART Divisor Latch High Register"]
pub mod dlh;
#[doc = "ier (rw) register accessor: an alias for `Reg<IER_SPEC>`"]
pub type IER = crate::Reg<ier::IER_SPEC>;
#[doc = "UART Interrupt Enable Register"]
pub mod ier;
#[doc = "iir (r) register accessor: an alias for `Reg<IIR_SPEC>`"]
pub type IIR = crate::Reg<iir::IIR_SPEC>;
#[doc = "UART Interrupt Identity Register"]
pub mod iir;
#[doc = "fcr (w) register accessor: an alias for `Reg<FCR_SPEC>`"]
pub type FCR = crate::Reg<fcr::FCR_SPEC>;
#[doc = "UART FIFO Control Register"]
pub mod fcr;
#[doc = "lcr (rw) register accessor: an alias for `Reg<LCR_SPEC>`"]
pub type LCR = crate::Reg<lcr::LCR_SPEC>;
#[doc = "UART Line Control Register"]
pub mod lcr;
#[doc = "mcr (rw) register accessor: an alias for `Reg<MCR_SPEC>`"]
pub type MCR = crate::Reg<mcr::MCR_SPEC>;
#[doc = "UART Modem Control Register"]
pub mod mcr;
#[doc = "lsr (rw) register accessor: an alias for `Reg<LSR_SPEC>`"]
pub type LSR = crate::Reg<lsr::LSR_SPEC>;
#[doc = "UART Line Status Register"]
pub mod lsr;
#[doc = "msr (rw) register accessor: an alias for `Reg<MSR_SPEC>`"]
pub type MSR = crate::Reg<msr::MSR_SPEC>;
#[doc = "UART Modem Status Register"]
pub mod msr;
#[doc = "sch (rw) register accessor: an alias for `Reg<SCH_SPEC>`"]
pub type SCH = crate::Reg<sch::SCH_SPEC>;
#[doc = "UART Scratch Register"]
pub mod sch;
