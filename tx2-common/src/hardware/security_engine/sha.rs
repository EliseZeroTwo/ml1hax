use crate::{
    dsb_ish,
    hardware::security_engine::do_operation,
    mmio::{self, mmio_read, mmio_write},
    utils::clear_and_invalidate_cache_for_address_range,
};

macro_rules! read_word_to_u8_slice {
    ($base:expr, $reg:expr, $slice:expr, $sbase:expr) => {{
        let word = mmio_read($base, $reg);
        $slice[$sbase + 0] = (word >> 0x18) as u8;
        $slice[$sbase + 1] = (word >> 0x10) as u8;
        $slice[$sbase + 2] = (word >> 0x8) as u8;
        $slice[$sbase + 3] = word as u8;
    }};
}

pub fn do_sha256(
    se_base: usize,
    input: *const u8,
    input_len: usize,
) -> [u8; 0x20] {
    mmio_write(se_base, mmio::security_engine::CONFIG, 0x5003004);
    mmio_write(se_base, mmio::security_engine::SHA_CONFIG, 1);

    mmio_write(se_base, mmio::security_engine::HASH_RESULT_0, 0);
    mmio_write(se_base, mmio::security_engine::HASH_RESULT_1, 0);
    mmio_write(se_base, mmio::security_engine::HASH_RESULT_2, 0);
    mmio_write(se_base, mmio::security_engine::HASH_RESULT_3, 0);
    mmio_write(se_base, mmio::security_engine::HASH_RESULT_4, 0);
    mmio_write(se_base, mmio::security_engine::HASH_RESULT_5, 0);
    mmio_write(se_base, mmio::security_engine::HASH_RESULT_6, 0);
    mmio_write(se_base, mmio::security_engine::HASH_RESULT_7, 0);

    let size_in_bits = (input_len * 8) as u32;
    mmio_write(
        se_base,
        mmio::security_engine::SHA_MSG_LENGTH_0,
        size_in_bits,
    );
    mmio_write(se_base, mmio::security_engine::SHA_MSG_LENGTH_1, 0);
    mmio_write(se_base, mmio::security_engine::SHA_MSG_LENGTH_2, 0);
    mmio_write(se_base, mmio::security_engine::SHA_MSG_LENGTH_3, 0);
    mmio_write(se_base, mmio::security_engine::SHA_MSG_LEFT_0, size_in_bits);
    mmio_write(se_base, mmio::security_engine::SHA_MSG_LEFT_1, 0);
    mmio_write(se_base, mmio::security_engine::SHA_MSG_LEFT_2, 0);
    mmio_write(se_base, mmio::security_engine::SHA_MSG_LEFT_3, 0);

    clear_and_invalidate_cache_for_address_range(input as u64, input_len as u64);
    dsb_ish!();

    do_operation(
        se_base,
        core::ptr::null_mut(),
        0,
        input,
        input_len as u32,
    );

    let mut out = [0u8; 0x20];

    for (iter, reg) in [
        mmio::security_engine::HASH_RESULT_0,
        mmio::security_engine::HASH_RESULT_1,
        mmio::security_engine::HASH_RESULT_2,
        mmio::security_engine::HASH_RESULT_3,
        mmio::security_engine::HASH_RESULT_4,
        mmio::security_engine::HASH_RESULT_5,
        mmio::security_engine::HASH_RESULT_6,
        mmio::security_engine::HASH_RESULT_7,
    ]
    .into_iter()
    .enumerate()
    {
        read_word_to_u8_slice!(se_base, reg, out, iter * 4);
    }

    out
}
