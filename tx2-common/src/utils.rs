use core::arch::asm;

use crate::{
    dc_isw, dsb_ish, hardware::tmr::read_usec_cntr, ic_iallu, isb, read_spr, write_spr
};

#[inline(always)]
pub fn cpuid() -> u8 {
    let mut mpidr: u64;

    unsafe {
        asm!(
            "MRS {output}, MPIDR_EL1",
            output = out(reg) mpidr
        );
    }

    (mpidr & 0xFF) as u8
}

#[inline(always)]
pub fn invalidate_unified_and_data_caches() {
    let clidr_el1 = read_spr!("CLIDR_EL1");
    let cache_coherancy_level = (clidr_el1 >> 0x18) & 0b111;

    for current_level in 0..cache_coherancy_level {
        let new_level_shifted = current_level << 1;
        write_spr!("CSSELR_EL1", new_level_shifted);
        isb!();

        let current_cache_size_id = read_spr!("CCSIDR_EL1");
        let associativity = ((current_cache_size_id >> 3) & 0x3FF) as u32;
        let line_size = (current_cache_size_id & 0b111) + 4;
        let number_of_sets_in_cache = ((current_cache_size_id >> 13) & 0x7FFF) + 1;

        let leading_zeros_of_associativity = (associativity + 1).leading_zeros() + 1;

        for current_associativity in 0..associativity {
            let shifted_val = ((current_associativity << leading_zeros_of_associativity) as u64)
                | new_level_shifted;
            for iter in 0..number_of_sets_in_cache {
                dc_isw!((iter << line_size) | shifted_val);
            }
        }
    }

    dsb_ish!();
    ic_iallu!();
    dsb_ish!();
    isb!();
}

#[inline(always)]
pub fn clear_and_invalidate_cache_for_address_range(start: u64, length: u64) {
    let end = align_up(start + length as u64, 0x40);
    let mut ptr = start;
    while ptr < end {
        dc_isw!(ptr);
        ptr += 0x40;
    }
}

#[inline(always)]
pub const fn align_up(value: u64, alignment: u64) -> u64 {
    (value | (alignment - 1)) + 1
}

pub fn usleep(usecs: u32) {
    let start = read_usec_cntr();
    let (end, overflows) = start.overflowing_add(usecs);

    if overflows {
        while {
            let now = read_usec_cntr();
            start < now || now < end
        } {}  
    } else {
        let mut last = read_usec_cntr();
        while (last - start) < usecs {
            last = read_usec_cntr();
        }
    }
}

pub fn endian_flip(input: u32) -> u32 {
    ((input & 0x0000_00FF) << (3 * 8))
        | ((input & 0x0000_FF00) << 8)
        | ((input & 0x00FF_0000) >> 8)
        | ((input & 0xFF00_0000) >> (3 * 8))
}

pub fn align_buffer(input: &mut [u8], alignment: u64, needed_size: u64) -> &mut [u8] {
    let start_offset =
        (align_up(input.as_ptr() as u64, alignment) - input.as_ptr() as u64) as usize;
    &mut input[start_offset..(start_offset + needed_size as usize)]
}

#[inline(always)]
pub fn translate_address<const LMASK: u64, T>(input: *const T) -> u32 {
    let at_output: u64;
    unsafe {
        asm!(
            "AT S1E3R, {0}",
            "MRS {1}, PAR_EL1",
            in(reg) input,
            out(reg) at_output
        )
    }

    ((at_output & 0xfffff000) | (input as u64 & LMASK)) as u32
}
