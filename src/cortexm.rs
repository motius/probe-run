//! ARM Cortex-M specific constants

use std::{mem, ops::Range};

pub(crate) const ADDRESS_SIZE: u8 = mem::size_of::<u32>() as u8;
pub(crate) const EXC_RETURN_MARKER: u32 = 0xFFFF_FFE0;
const THUMB_BIT: u32 = 1;
// According to the ARM Cortex-M Reference Manual RAM memory must be located in this address range
// (vendors still place e.g. Core-Coupled RAM outside this address range)
pub(crate) const VALID_RAM_ADDRESS: Range<u32> = 0x2000_0000..0x4000_0000;

pub(crate) fn clear_thumb_bit(addr: u32) -> u32 {
    addr & !THUMB_BIT
}

/// Checks if PC is the HardFault handler
// XXX may want to relax this to cover the whole PC range of the `HardFault` handler
pub(crate) fn is_hard_fault(pc: u32, vector_table: &VectorTable) -> bool {
    subroutine_eq(pc, vector_table.hard_fault)
}

pub(crate) fn is_thumb_bit_set(addr: u32) -> bool {
    addr & THUMB_BIT == THUMB_BIT
}

pub(crate) fn set_thumb_bit(addr: u32) -> u32 {
    addr | THUMB_BIT
}

/// Checks if two subroutine addresses are equivalent by first clearing their `THUMB_BIT`
pub(crate) fn subroutine_eq(addr1: u32, addr2: u32) -> bool {
    addr1 & !THUMB_BIT == addr2 & !THUMB_BIT
}

/// The contents of the vector table
#[derive(Debug)]
pub(crate) struct VectorTable {
    pub(crate) location: u32,
    // entry 0
    pub(crate) initial_stack_pointer: u32,
    // entry 1: Reset handler
    pub(crate) reset: u32,
    // entry 3: HardFault handler
    pub(crate) hard_fault: u32,
}
