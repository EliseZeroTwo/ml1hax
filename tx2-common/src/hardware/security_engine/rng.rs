use crate::{
    mmio::{self, mmio_write},
    utils::align_buffer,
};

use super::{do_operation, perform_crypto_single_block_operation};

pub fn generate_random(se_base: usize, output: &mut [u8]) {
    if !output.is_empty() {
        let output_len = output.len();

        let block_count = output_len / 0x10;
        let aligned_len = block_count * 0x10;
        let diff = output_len - aligned_len;

        mmio_write(se_base, mmio::security_engine::CONFIG, 0x2000);
        mmio_write(se_base, mmio::security_engine::CRYPTO_CONFIG, 0x108);
        mmio_write(se_base, mmio::security_engine::RNG_CONFIG, 4);

        if aligned_len as u32 != 0 {
            mmio_write(
                se_base,
                mmio::security_engine::CRYPTO_LAST_BLOCK,
                block_count as u32 - 1,
            );
            do_operation(
                se_base,
                output.as_mut_ptr(),
                aligned_len as u32,
                core::ptr::null(),
                0,
            );
        }

        if diff != 0 {
            let new_out_buf = &mut output[aligned_len..];
            perform_crypto_single_block_operation(
                se_base,
                new_out_buf.as_mut_ptr(),
                diff as u32,
                core::ptr::null(),
                0,
            );
        }
    }
}

pub fn setup_rng(se_base: usize) {
    mmio_write(se_base, mmio::security_engine::RNG_SRC_CONFIG, 3);
    mmio_write(se_base, mmio::security_engine::RNG_RESEED_INTERVAL, 0x11171);
    mmio_write(se_base, mmio::security_engine::CONFIG, 0x2000);
    mmio_write(se_base, mmio::security_engine::CRYPTO_CONFIG, 0x108);
    mmio_write(se_base, mmio::security_engine::RNG_CONFIG, 5);
    mmio_write(se_base, mmio::security_engine::CRYPTO_LAST_BLOCK, 0);

    let mut output = [0u8; 0x50];
    let aoutput = align_buffer(&mut output, 0x40, 0x10);
    do_operation(
        se_base,
        aoutput.as_mut_ptr(),
        0x10,
        core::ptr::null(),
        0,
    );
}

pub fn fill_aes_register(se_base: usize, keyslot: u32) {
    mmio_write(se_base, mmio::security_engine::CONFIG, 0x2000 | 0x8);
    mmio_write(se_base, mmio::security_engine::CRYPTO_CONFIG, 0x108);
    mmio_write(se_base, mmio::security_engine::RNG_CONFIG, 4);
    mmio_write(
        se_base,
        mmio::security_engine::CRYPTO_KEYTABLE_DST,
        keyslot << 8,
    );
    mmio_write(se_base, mmio::security_engine::CRYPTO_LAST_BLOCK, 0);
    do_operation(
        se_base,
        core::ptr::null_mut(),
        0,
        core::ptr::null(),
        0,
    );
    mmio_write(
        se_base,
        mmio::security_engine::CRYPTO_KEYTABLE_DST,
        1 | ((0xffffff & keyslot) << 8),
    );
    do_operation(
        se_base,
        core::ptr::null_mut(),
        0,
        core::ptr::null(),
        0,
    );
}

pub fn do_something_tm(se_base: usize) {
    mmio_write(se_base, mmio::security_engine::CONFIG, 0x2000 | 0xc);
    mmio_write(se_base, mmio::security_engine::CRYPTO_CONFIG, 0x108);
    mmio_write(se_base, mmio::security_engine::RNG_CONFIG, 6);
    mmio_write(se_base, mmio::security_engine::CRYPTO_LAST_BLOCK, 0);
    do_operation(
        se_base,
        core::ptr::null_mut(),
        0,
        core::ptr::null(),
        0,
    );
}
