use crate::{
    dsb_ish,
    mmio::{self, mmio_and, mmio_or, mmio_read, mmio_write},
    utils::{
        align_buffer, clear_and_invalidate_cache_for_address_range,
        invalidate_unified_and_data_caches, translate_address, usleep,
    },
};

pub mod aes;
// pub mod rng;
// pub mod rsa;
// pub mod sha;

#[macro_export]
macro_rules! verify_alignment {
    ($a:expr, $func:literal) => {
        if $a as u32 & 0xFFFF_FFC0 != $a as u32 {
            loop {}
        }
    };
}

pub fn is_idle(base: usize) -> bool {
    mmio_read(base, mmio::security_engine::STATUS) & 0b111 == 0
}

pub fn panic_if_se_isnt_idle(se_base: usize) {
    if !is_idle(se_base) {
        loop {}
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct SeLLEntry {
    pub unk: u32,
    pub translated_addr: u32,
    pub size: u32,
}

impl SeLLEntry {
    pub fn new(unk: u32, vaddr: *const u8, size: u32) -> Self {
        if vaddr.is_null() {
            return Self {
                unk: 0,
                translated_addr: 0,
                size: 0,
            };
        }

        Self {
            unk,
            translated_addr: translate_address::<0xFFF, _>(vaddr),
            size,
        }
    }
}

pub fn do_operation(
    se_base: usize,
    output: *mut u8,
    output_size: u32,
    input: *const u8,
    input_size: u32,
) {
    let input_ll = SeLLEntry::new(0, input, input_size);
    let output_ll = SeLLEntry::new(0, output, output_size);

    if &input_ll as *const _ as u64 & 0xFFFF_FFF8 != &input_ll as *const _ as u64 {
        loop {}
    }

    if &output_ll as *const _ as u64 & 0xFFFF_FFF8 != &output_ll as *const _ as u64 {
        loop {}
    }

    clear_and_invalidate_cache_for_address_range(&input_ll as *const _ as u64, 0xC);
    clear_and_invalidate_cache_for_address_range(&output_ll as *const _ as u64, 0xC);
    dsb_ish!();

    // N don't do this but it's needed otherwise our stack "breaks".
    invalidate_unified_and_data_caches();

    mmio_write(
        se_base,
        mmio::security_engine::IN_LL_ADDR,
        translate_address::<0xFF8, _>(&input_ll),
    );
    mmio_write(
        se_base,
        mmio::security_engine::OUT_LL_ADDR,
        translate_address::<0xFF8, _>(&output_ll),
    );

    mmio_write(
        se_base,
        mmio::security_engine::ERR_STATUS,
        mmio_read(se_base, mmio::security_engine::ERR_STATUS),
    );
    mmio_write(
        se_base,
        mmio::security_engine::INT_STATUS,
        mmio_read(se_base, mmio::security_engine::INT_STATUS),
    );
    mmio_write(se_base, mmio::security_engine::OPERATION, 1);

    while mmio_read(se_base, mmio::security_engine::INT_STATUS) & 0x10 == 0 {
        usleep(500);
    }

    if mmio_read(se_base, mmio::security_engine::INT_STATUS) & 0x10000 != 0
        || mmio_read(se_base, mmio::security_engine::STATUS) & 3 != 0
        || mmio_read(se_base, mmio::security_engine::ERR_STATUS) != 0
    {
        loop {}
    }

    // N don't do this but otherwise we don't see the SE output for some reason. Figure out why later.
    invalidate_unified_and_data_caches();
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn perform_crypto_single_block_operation(
    se_base: usize,
    output: *mut u8,
    output_size: u32,
    input: *const u8,
    input_size: u32,
) {
    let mut buffer = [0u8; 0x50];
    let aligned = align_buffer(&mut buffer, 0x40, 0x10);

    if input_size > 0x10 || output_size > 0x10 {
        loop {}
    }

    verify_alignment!(aligned.as_ptr(), "pcsbo");

    mmio_write(se_base, mmio::security_engine::CRYPTO_LAST_BLOCK, 0);

    if !input.is_null() {
        unsafe {
            core::ptr::copy(input, aligned.as_mut_ptr(), input_size as usize);
        }
    }
    clear_and_invalidate_cache_for_address_range(aligned.as_ptr() as u64, 0x10);
    dsb_ish!();

    do_operation(
        se_base,
        aligned.as_mut_ptr(),
        0x10,
        aligned.as_ptr(),
        0x10,
    );
    dsb_ish!();

    clear_and_invalidate_cache_for_address_range(aligned.as_mut_ptr() as u64, 0x10);
    dsb_ish!();

    if !output.is_null() {
        unsafe {
            core::ptr::copy(aligned.as_ptr(), output, output_size as usize);
        }
    }
}

pub fn lock(base: usize) {
    for x in 0..0x10 {
        mmio_write(
            base,
            mmio::security_engine::CRYPTO_KEYTABLE_ACCESS_0 + (x * 4),
            0,
        );
    }

    mmio_write(base, mmio::security_engine::RSA_KEYTABLE_ACCESS_0, 0);
    mmio_write(base, mmio::security_engine::RSA_KEYTABLE_ACCESS_1, 0);

    mmio_write(base, mmio::security_engine::CRYPTO_SECURITY_PERKEY, 0);
    mmio_write(base, mmio::security_engine::RSA_SECURITY_PERKEY, 0);

    mmio_and(base, mmio::security_engine::SECURITY_CONTROL, 0xfffffffb);
    mmio_write(base, mmio::security_engine::SECURITY_CONTROL, 2);
}

pub fn set_security_state(base: usize, a: bool) {
    match a {
        true => mmio_and(base, mmio::security_engine::SECURITY_CONTROL, !(1 << 16)),
        false => mmio_or(base, mmio::security_engine::SECURITY_CONTROL, 1 << 16),
    }
    mmio_read(base, mmio::security_engine::STATUS);
}
