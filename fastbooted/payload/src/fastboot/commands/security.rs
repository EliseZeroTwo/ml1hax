use tx2_common::mmio::mmio_read;

use crate::{
    fastboot::{
        FastbootCommandHandlerRes, externs::transport_usbf_send, fastboot_data, fastboot_info,
        fastboot_okay,
    },
    try_something,
};

fn aes_set_keyslot_partial(keyslot: u32, idx: u32, val: u32) {
    tx2_common::mmio::mmio_write(
        tx2_common::mmio::security_engine::SE_BASE,
        tx2_common::mmio::security_engine::CRYPTO_KEYTABLE_ADDR,
        (keyslot << 4) | idx as u32,
    );
    tx2_common::mmio::mmio_write(
        tx2_common::mmio::security_engine::SE_BASE,
        tx2_common::mmio::security_engine::CRYPTO_KEYTABLE_DATA,
        val,
    );
}

fn aes_set_iv_partial(keyslot: u32, updated: bool, idx: u32, val: u32) {
    let updated = match updated {
        true => 1u32,
        false => 0u32,
    };

    tx2_common::mmio::mmio_write(
        tx2_common::mmio::security_engine::SE_BASE,
        tx2_common::mmio::security_engine::CRYPTO_KEYTABLE_ADDR,
        (keyslot << 4) | (1 << 3) | (updated << 2) | idx as u32,
    );
    tx2_common::mmio::mmio_write(
        tx2_common::mmio::security_engine::SE_BASE,
        tx2_common::mmio::security_engine::CRYPTO_KEYTABLE_DATA,
        val,
    );
}

fn aes_read_keyslot_partial(keyslot: u32, idx: u32) -> u32 {
    tx2_common::mmio::mmio_write(
        tx2_common::mmio::security_engine::SE_BASE,
        tx2_common::mmio::security_engine::CRYPTO_KEYTABLE_ADDR,
        (keyslot << 4) | idx as u32,
    );
    tx2_common::mmio::mmio_read(
        tx2_common::mmio::security_engine::SE_BASE,
        tx2_common::mmio::security_engine::CRYPTO_KEYTABLE_DATA,
    )
}

fn aes_read_iv_partial(keyslot: u32, updated: bool, idx: u32) -> u32 {
    let updated = match updated {
        true => 1u32,
        false => 0u32,
    };

    tx2_common::mmio::mmio_write(
        tx2_common::mmio::security_engine::SE_BASE,
        tx2_common::mmio::security_engine::CRYPTO_KEYTABLE_ADDR,
        (keyslot << 4) | (1 << 3) | (updated << 2) | idx as u32,
    );
    tx2_common::mmio::mmio_read(
        tx2_common::mmio::security_engine::SE_BASE,
        tx2_common::mmio::security_engine::CRYPTO_KEYTABLE_DATA,
    )
}

fn fastboot_se_hax_dump_vectors_inner(keyslot: u8) -> FastbootCommandHandlerRes {
    aes_set_iv_partial(keyslot as u32, false, 0, 0);
    aes_set_iv_partial(keyslot as u32, false, 1, 0);
    aes_set_iv_partial(keyslot as u32, false, 2, 0);
    aes_set_iv_partial(keyslot as u32, false, 3, 0);
    aes_set_iv_partial(keyslot as u32, false, 4, 0);
    aes_set_iv_partial(keyslot as u32, false, 5, 0);
    aes_set_iv_partial(keyslot as u32, false, 6, 0);
    aes_set_iv_partial(keyslot as u32, false, 7, 0);
    aes_set_iv_partial(keyslot as u32, true, 0, 0);
    aes_set_iv_partial(keyslot as u32, true, 1, 0);
    aes_set_iv_partial(keyslot as u32, true, 2, 0);
    aes_set_iv_partial(keyslot as u32, true, 3, 0);
    aes_set_iv_partial(keyslot as u32, true, 4, 0);
    aes_set_iv_partial(keyslot as u32, true, 5, 0);
    aes_set_iv_partial(keyslot as u32, true, 6, 0);
    aes_set_iv_partial(keyslot as u32, true, 7, 0);

    let start_1 = [0u8; 0x10];
    let mut expected_0123 = [0u8; 0x10];
    let mut expected_012 = [0u8; 0x10];
    let mut expected_01 = [0u8; 0x10];
    let mut expected_0 = [0u8; 0x10];
    let mut expected_empty = [0u8; 0x10];

    try_something!(
        crate::fastboot::externs::se_aes_encrypt_decrypt(
            keyslot,
            0,
            true,
            1,
            &start_1,
            &mut expected_0123,
            true
        ),
        b"Encrypt1 failed"
    );
    aes_set_keyslot_partial(keyslot as u32, 3, 0u32);
    try_something!(
        crate::fastboot::externs::se_aes_encrypt_decrypt(
            keyslot,
            0,
            true,
            1,
            &start_1,
            &mut expected_012,
            true
        ),
        b"Encrypt1 failed"
    );
    aes_set_keyslot_partial(keyslot as u32, 2, 0u32);
    try_something!(
        crate::fastboot::externs::se_aes_encrypt_decrypt(
            keyslot,
            0,
            true,
            1,
            &start_1,
            &mut expected_01,
            true
        ),
        b"Encrypt1 failed"
    );
    aes_set_keyslot_partial(keyslot as u32, 1, 0u32);
    try_something!(
        crate::fastboot::externs::se_aes_encrypt_decrypt(
            keyslot,
            0,
            true,
            1,
            &start_1,
            &mut expected_0,
            true
        ),
        b"Encrypt1 failed"
    );
    aes_set_keyslot_partial(keyslot as u32, 0, 0u32);
    try_something!(
        crate::fastboot::externs::se_aes_encrypt_decrypt(
            keyslot,
            0,
            true,
            1,
            &start_1,
            &mut expected_empty,
            true
        ),
        b"Encrypt1 failed"
    );

    let mut buffer = [0u8; 0x60];
    buffer[..0x10].copy_from_slice(&start_1);
    buffer[0x10..0x20].copy_from_slice(&expected_empty);
    buffer[0x20..0x30].copy_from_slice(&expected_0);
    buffer[0x30..0x40].copy_from_slice(&expected_01);
    buffer[0x40..0x50].copy_from_slice(&expected_012);
    buffer[0x50..0x60].copy_from_slice(&expected_0123);

    try_something!(fastboot_data(0x60));
    try_something!(transport_usbf_send(&buffer));

    FastbootCommandHandlerRes::Continue
}

pub fn fastboot_se_hax_dump_vectors(_arg: &[u8]) -> FastbootCommandHandlerRes {
    for keyslot in 0..16 {
        if fastboot_se_hax_dump_vectors_inner(keyslot) == FastbootCommandHandlerRes::DropDevice {
            return FastbootCommandHandlerRes::DropDevice;
        }
    }

    try_something!(fastboot_okay(b""));
    FastbootCommandHandlerRes::Continue
}

pub fn fastboot_se_hax_validate(_arg: &[u8]) -> FastbootCommandHandlerRes {
    let keyslot = 13;
    let start = [0u8; 0x10];
    for _ in 0..2 {
        let mut expected = [0u8; 0x10];
        try_something!(
            crate::fastboot::externs::se_aes_encrypt_decrypt(
                keyslot,
                2,
                true,
                1,
                &start,
                &mut expected,
                true
            ),
            b"Encrypt1 failed"
        );

        let mut buffer = [0u8; 0x10];

        aes_set_keyslot_partial(keyslot as u32, 0, 0xE614_E3D1);
        try_something!(
            crate::fastboot::externs::se_aes_encrypt_decrypt(
                keyslot,
                2,
                true,
                1,
                &start,
                &mut buffer,
                true
            ),
            b"Encrypt0-1 failed"
        );

        if buffer == expected {
            try_something!(fastboot_info(b"Key idx0 matched!"));
            // found_key[0..4].copy_from_slice(&0xE614_E3D1u32.to_le_bytes());
            // try_something!(fastboot_data(0x10));
            // try_something!(transport_usbf_send(&found_key));
        } else {
            try_something!(fastboot_info(b"Key idx0 did not match :c"));
        }

        aes_set_keyslot_partial(keyslot as u32, 1, 0x5916_D477);
        try_something!(
            crate::fastboot::externs::se_aes_encrypt_decrypt(
                keyslot,
                2,
                true,
                1,
                &start,
                &mut buffer,
                true
            ),
            b"Encrypt0-2 failed"
        );

        if buffer == expected {
            try_something!(fastboot_info(b"Key idx1 matched!"));
            // found_key[4..8].copy_from_slice(&0x5916_D477u32.to_le_bytes());
            // try_something!(fastboot_data(0x10));
            // try_something!(transport_usbf_send(&found_key));
        } else {
            try_something!(fastboot_info(b"Key idx1 did not match :c"));
        }

        aes_set_keyslot_partial(keyslot as u32, 2, 0x8F454A44);
        try_something!(
            crate::fastboot::externs::se_aes_encrypt_decrypt(
                keyslot,
                2,
                true,
                1,
                &start,
                &mut buffer,
                true
            ),
            b"Encrypt0-3 failed"
        );

        if buffer == expected {
            try_something!(fastboot_info(b"Key idx2 matched!"));
            // found_key[4..8].copy_from_slice(&0x5916_D477u32.to_le_bytes());
            // try_something!(fastboot_data(0x10));
            // try_something!(transport_usbf_send(&found_key));
        } else {
            try_something!(fastboot_info(b"Key idx2 did not match :c"));
        }

        aes_set_iv_partial(keyslot as u32, false, 0, 0);
        aes_set_iv_partial(keyslot as u32, false, 1, 0);
        aes_set_iv_partial(keyslot as u32, false, 2, 0);
        aes_set_iv_partial(keyslot as u32, false, 3, 0);
        aes_set_iv_partial(keyslot as u32, true, 0, 0);
        aes_set_iv_partial(keyslot as u32, true, 1, 0);
        aes_set_iv_partial(keyslot as u32, true, 2, 0);
        aes_set_iv_partial(keyslot as u32, true, 3, 0);
    }
    try_something!(fastboot_okay(b""));

    FastbootCommandHandlerRes::Continue
}

pub fn fastboot_test_se_hax(_arg: &[u8]) -> FastbootCommandHandlerRes {
    let start = [0u8; 0x10];
    let mut out_b1 = [0u8; 0x10];
    let mut out_b2 = [0u8; 0x10];
    let mut out_b3 = [0u8; 0x10];
    let mut out_b4 = [0u8; 0x10];

    aes_set_keyslot_partial(8, 0, 0);
    aes_set_keyslot_partial(8, 1, 0xFFFFFFFF);
    aes_set_keyslot_partial(8, 2, 0);
    aes_set_keyslot_partial(8, 3, 0);

    try_something!(
        crate::fastboot::externs::se_aes_encrypt_decrypt(8, 2, true, 1, &start, &mut out_b1, true),
        b"Encrypt1 failed"
    );
    try_something!(
        crate::fastboot::externs::se_aes_encrypt_decrypt(
            8,
            2,
            true,
            1,
            &out_b1,
            &mut out_b2,
            false
        ),
        b"Encrypt2 failed"
    );

    aes_set_keyslot_partial(8, 1, 0);
    try_something!(
        crate::fastboot::externs::se_aes_encrypt_decrypt(8, 2, true, 1, &start, &mut out_b3, true),
        b"Encrypt3 failed"
    );

    aes_set_keyslot_partial(8, 1, 0xFFFFFFFF);
    try_something!(
        crate::fastboot::externs::se_aes_encrypt_decrypt(8, 2, true, 1, &start, &mut out_b4, true),
        b"Encrypt4 failed"
    );

    let res = if out_b1 == out_b4 && out_b1 != out_b3 {
        fastboot_info(b"This SE is vulnerable!")
    } else {
        fastboot_info(b"This SE is not vulnerable!")
    };

    if res.is_err() {
        return FastbootCommandHandlerRes::DropDevice;
    }

    if fastboot_data(0x40).is_err() {
        return FastbootCommandHandlerRes::DropDevice;
    }

    let mut write_buffer = [0u8; 0x40];
    write_buffer[..0x10].copy_from_slice(&out_b1);
    write_buffer[0x10..0x20].copy_from_slice(&out_b2);
    write_buffer[0x20..0x30].copy_from_slice(&out_b3);
    write_buffer[0x30..0x40].copy_from_slice(&out_b4);

    if transport_usbf_send(&write_buffer).is_err() {
        return FastbootCommandHandlerRes::DropDevice;
    }

    if fastboot_okay(b"").is_err() {
        return FastbootCommandHandlerRes::DropDevice;
    }

    FastbootCommandHandlerRes::Continue
}

pub fn fastboot_read_sysram(_arg: &[u8]) -> FastbootCommandHandlerRes {
    let src = [0u8; 0x10];
    let mut dst = [0u8; 0x10];
    let keyslot = 1u32;
    aes_set_iv_partial(keyslot as u32, false, 0, 0);
    aes_set_iv_partial(keyslot as u32, false, 1, 0);
    aes_set_iv_partial(keyslot as u32, false, 2, 0);
    aes_set_iv_partial(keyslot as u32, false, 3, 0);
    aes_set_iv_partial(keyslot as u32, false, 4, 0);
    aes_set_iv_partial(keyslot as u32, false, 5, 0);
    aes_set_iv_partial(keyslot as u32, false, 6, 0);
    aes_set_iv_partial(keyslot as u32, false, 7, 0);
    aes_set_iv_partial(keyslot as u32, true, 0, 0);
    aes_set_iv_partial(keyslot as u32, true, 1, 0);
    aes_set_iv_partial(keyslot as u32, true, 2, 0);
    aes_set_iv_partial(keyslot as u32, true, 3, 0);
    aes_set_iv_partial(keyslot as u32, true, 4, 0);
    aes_set_iv_partial(keyslot as u32, true, 5, 0);
    aes_set_iv_partial(keyslot as u32, true, 6, 0);
    aes_set_iv_partial(keyslot as u32, true, 7, 0);
    aes_set_keyslot_partial(keyslot, 0, 0u32);
    aes_set_keyslot_partial(keyslot, 1, 0u32);
    aes_set_keyslot_partial(keyslot, 2, 0u32);
    aes_set_keyslot_partial(keyslot, 3, 0u32);
    try_something!(
        crate::fastboot::externs::se_aes_encrypt_decrypt_raw(
            keyslot as u8,
            0,
            true,
            1,
            src.as_ptr(),
            // 0x30000000 as *mut u8,
            dst.as_mut_ptr(),
            true
        ),
        b"Encrypt failed"
    );

    try_something!(fastboot_data(0x10));
    try_something!(transport_usbf_send(&dst));
    try_something!(fastboot_okay(b""));

    FastbootCommandHandlerRes::Continue
}
