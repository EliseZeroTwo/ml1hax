use crate::{
    dsb_ish,
    hardware::security_engine::{do_operation, perform_crypto_single_block_operation},
    mmio::{self, mmio_and, mmio_or, mmio_write},
    utils::clear_and_invalidate_cache_for_address_range,
    verify_alignment,
};

pub fn clear_keyslot(se_base: usize, keyslot: u32) {
    if keyslot > 0x10 {
        loop {}
    }

    for x in 0..0x10 {
        mmio_write(
            se_base,
            mmio::security_engine::CRYPTO_KEYTABLE_ADDR,
            (keyslot << 4) | x,
        );
        mmio_write(se_base, mmio::security_engine::CRYPTO_KEYTABLE_DATA, 0);
    }
}

pub fn aes_set_keyslot_partial(se_base: usize, keyslot: u32, idx: u32, val: u32) {
    if keyslot > 0x10 {
        loop {}
    }

    mmio_write(
        se_base,
        mmio::security_engine::CRYPTO_KEYTABLE_ADDR,
        (keyslot << 4) | idx as u32,
    );
    mmio_write(
        se_base,
        mmio::security_engine::CRYPTO_KEYTABLE_DATA,
        val,
    );
}

pub fn aes_set_keyslot(se_base: usize, keyslot: u32, data: &[u8]) {
    if keyslot > 0x10 {
        loop {}
    }

    if data.len() != 0x10 {
        loop {}
    }

    for x in 0..0x10 {
        mmio_write(
            se_base,
            mmio::security_engine::CRYPTO_KEYTABLE_ADDR,
            (keyslot << 4) | x,
        );
        mmio_write(
            se_base,
            mmio::security_engine::CRYPTO_KEYTABLE_DATA,
            data[x as usize] as u32,
        );
    }
}

pub fn set_aes_keyslot_perms(se_base: usize, keyslot: u32, perms: u32) {
    if keyslot > 0x10 {
        loop {}
    }

    if (perms & 0xFFFF_FF7F) != 0 {
        mmio_write(
            se_base,
            match keyslot {
                0 => mmio::security_engine::CRYPTO_KEYTABLE_ACCESS_0,
                1 => mmio::security_engine::CRYPTO_KEYTABLE_ACCESS_1,
                2 => mmio::security_engine::CRYPTO_KEYTABLE_ACCESS_2,
                3 => mmio::security_engine::CRYPTO_KEYTABLE_ACCESS_3,
                4 => mmio::security_engine::CRYPTO_KEYTABLE_ACCESS_4,
                5 => mmio::security_engine::CRYPTO_KEYTABLE_ACCESS_5,
                6 => mmio::security_engine::CRYPTO_KEYTABLE_ACCESS_6,
                7 => mmio::security_engine::CRYPTO_KEYTABLE_ACCESS_7,
                8 => mmio::security_engine::CRYPTO_KEYTABLE_ACCESS_8,
                9 => mmio::security_engine::CRYPTO_KEYTABLE_ACCESS_9,
                10 => mmio::security_engine::CRYPTO_KEYTABLE_ACCESS_10,
                11 => mmio::security_engine::CRYPTO_KEYTABLE_ACCESS_11,
                12 => mmio::security_engine::CRYPTO_KEYTABLE_ACCESS_12,
                13 => mmio::security_engine::CRYPTO_KEYTABLE_ACCESS_13,
                14 => mmio::security_engine::CRYPTO_KEYTABLE_ACCESS_14,
                15 => mmio::security_engine::CRYPTO_KEYTABLE_ACCESS_15,
                _ => unreachable!(),
            },
            !perms,
        );
    }

    if (perms & 0x80) != 0 {
        mmio_and(
            se_base,
            mmio::security_engine::CRYPTO_SECURITY_PERKEY,
            !(1 << keyslot),
        );
    }
}

fn set_aes_ctr(se_base: usize, ctr: &[u8]) {
    let ctr: &[u8; 0x10] = match ctr.try_into() {
        Ok(ctr) => ctr,
        Err(_) => loop {},
    };

    // Have to have a *little* cursedness
    let ctr: &[u32; 4] = unsafe { core::mem::transmute(ctr) };

    mmio_write(se_base, mmio::security_engine::CRYPTO_LINEAR_CTR_0, ctr[0]);
    mmio_write(se_base, mmio::security_engine::CRYPTO_LINEAR_CTR_1, ctr[1]);
    mmio_write(se_base, mmio::security_engine::CRYPTO_LINEAR_CTR_2, ctr[2]);
    mmio_write(se_base, mmio::security_engine::CRYPTO_LINEAR_CTR_3, ctr[3]);
}

pub fn aes_ctr_decrypt_with_wrapped_key(
    se_base: usize,
    output: &mut [u8],
    input: &[u8],
    wrapped_key: &[u8; 0x10],
    ctr: &[u8],
) {
    let input_len = input.len() as u64;
    let output_len = output.len() as u64;
    let wrapped_key_len = wrapped_key.len() as u64;

    clear_and_invalidate_cache_for_address_range(wrapped_key.as_ptr() as u64, wrapped_key_len);
    clear_and_invalidate_cache_for_address_range(input.as_ptr() as u64, input_len);
    clear_and_invalidate_cache_for_address_range(output.as_ptr() as u64, output_len);
    dsb_ish!();

    verify_alignment!(input.as_ptr(), "ACDWWK Input");
    verify_alignment!(output.as_ptr(), "ACDWWK Output");

    unwrap_key(se_base, 8, 12, wrapped_key);
    aes_ctr_decrypt(se_base, output, input, 8, ctr);
    clear_keyslot(se_base, 8);
    clear_and_invalidate_cache_for_address_range(output.as_ptr() as u64, output_len);
    dsb_ish!();
}

pub fn aes_ctr_decrypt(
    se_base: usize,
    output: &mut [u8],
    input: &[u8],
    keyslot: u32,
    ctr: &[u8],
) {
    let input_len = input.len() as u64;
    let output_len = output.len() as u64;
    if input_len == 0 {
        return;
    }

    if ctr.len() != 0x10
        || keyslot >= 0x10
        || input_len > u32::MAX as u64
        || output_len > u32::MAX as u64
    {
        loop {}
    }

    let input_len = input.len() as u32;
    let output_len = output.len() as u32;

    verify_alignment!(input.as_ptr(), "aes_ctr input");
    verify_alignment!(output.as_ptr(), "aes_ctr output");

    mmio_write(se_base, mmio::security_engine::SPARE, 1);
    mmio_write(se_base, mmio::security_engine::CONFIG, 1 << 12);
    mmio_write(
        se_base,
        mmio::security_engine::CRYPTO_CONFIG,
        0x91e | (keyslot << 0x18),
    );
    set_aes_ctr(se_base, ctr);

    let aligned_input_len = input_len & 0xFFFF_FFF0;
    let input_len_diff = input_len & 0xF;

    let mut written = false;

    if aligned_input_len != 0 {
        mmio_write(
            se_base,
            mmio::security_engine::CRYPTO_LAST_BLOCK,
            (input_len / 0x10) - 1,
        );

        do_operation(
            se_base,
            output.as_mut_ptr(),
            output_len,
            input.as_ptr(),
            aligned_input_len,
        );
        dsb_ish!();

        written = true;
    }

    if input_len_diff != 0 && aligned_input_len < output_len {
        let out_len_diff = output_len - aligned_input_len;

        let new_output = &mut output[aligned_input_len as usize..];
        let new_input = &input[aligned_input_len as usize..];

        perform_crypto_single_block_operation(
            se_base,
            new_output.as_mut_ptr(),
            input_len_diff.min(out_len_diff),
            new_input.as_ptr(),
            input_len_diff,
        );
        written = true;
    }

    if !written {
        loop {}
    }
}

pub fn unwrap_key(
    se_base: usize,
    keytable_dst: u32,
    keytable_src: u32,
    data: &[u8],
) {
    let data_len = data.len() as u64;
    if data_len > 0x20 || keytable_dst > 0x10 || keytable_src > 0x10 {
        loop {}
    }

    mmio_write(se_base, mmio::security_engine::CONFIG, 0x108);
    mmio_write(
        se_base,
        mmio::security_engine::CRYPTO_CONFIG,
        keytable_src << 24,
    );
    mmio_write(se_base, mmio::security_engine::CRYPTO_LAST_BLOCK, 0);
    mmio_write(
        se_base,
        mmio::security_engine::CRYPTO_KEYTABLE_DST,
        keytable_dst << 8,
    );

    let data_ptr = data.as_ptr();
    clear_and_invalidate_cache_for_address_range(data_ptr as u64, data_len);
    dsb_ish!();

    do_operation(
        se_base,
        core::ptr::null_mut(),
        0,
        data_ptr,
        data_len as u32,
    );
}

pub fn aes_operation(
    se_base: usize,
    output: &mut [u8],
    input: &[u8],
    config: u16,
    keyslot: u32,
) {
    if input.is_empty() {
        return;
    }

    if output.len() != 0x10 || input.len() != 0x10 || keyslot >= 0x10 {
        loop {}
    }

    verify_alignment!(input.as_ptr(), "aes_operation input");
    verify_alignment!(output.as_ptr(), "aes_operation output");

    mmio_write(se_base, mmio::security_engine::CONFIG, 0x1000);
    mmio_write(
        se_base,
        mmio::security_engine::CRYPTO_CONFIG,
        0x100 | (keyslot << 0x18),
    );
    mmio_or(
        se_base,
        mmio::security_engine::CONFIG,
        (config as u32) << 0x10,
    );
    mmio_write(se_base, mmio::security_engine::CRYPTO_LAST_BLOCK, 0);

    clear_and_invalidate_cache_for_address_range(input.as_ptr() as u64, 0x10);
    dsb_ish!();

    do_operation(
        se_base,
        output.as_mut_ptr(),
        0x10,
        input.as_ptr(),
        0x10,
    );
    dsb_ish!();

    clear_and_invalidate_cache_for_address_range(output.as_ptr() as u64, 0x10);
    dsb_ish!();
}

pub fn keytable_slot_by_masterkey_revision(
    se_base: usize,
    revision: u32,
) -> u32 {
    const PROD_MK_WRAPPED_KEY: [[u8; 0x10]; 2] = [
        [
            0xf5, 0x89, 0xc4, 0x0c, 0xf1, 0x9d, 0x34, 0x6d, 0x2f, 0xb1, 0x76, 0xc1, 0x8e, 0x2d,
            0xf0, 0xf8,
        ],
        [
            0xde, 0xcf, 0xeb, 0xeb, 0x10, 0xae, 0x74, 0xd8, 0xad, 0x7c, 0xf4, 0x9e, 0x62, 0xe0,
            0xe8, 0x72,
        ],
    ];

    if revision > 2 {
        return 0xc;
    }

    let index = match revision {
        0 => 0,
        x => x - 1,
    };

    let keytable_dst = 9;

    unwrap_key(
        se_base,
        keytable_dst,
        0xc,
        &PROD_MK_WRAPPED_KEY[index as usize],
    );

    keytable_dst
}

pub fn unwrap_key_to_memory(
    se_base: usize,
    keytable_src: u32,
    data: &[u8],
    out: &mut [u8],
) {
    if data.len() != 0x10 || out.len() != 0x10 || keytable_src > 0x10 {
        loop {}
    }

    mmio_write(se_base, mmio::security_engine::CONFIG, 0x100);
    mmio_write(
        se_base,
        mmio::security_engine::CRYPTO_CONFIG,
        keytable_src << 24,
    );

    let data_ptr = data.as_ptr();
    clear_and_invalidate_cache_for_address_range(data_ptr as u64, 0x10);
    dsb_ish!();

    perform_crypto_single_block_operation(
        se_base,
        out.as_mut_ptr(),
        0x10,
        data_ptr,
        0x10,
    );
}
