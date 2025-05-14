use crate::mmio::{self, mmio_and, mmio_write};

pub fn set_rsa_keyslot_perms(se_base: usize, keyslot: u32, perms: u32) {
    if keyslot > 2 {
        loop {}
    }

    if (perms & 0xFFFF_FF7F) != 0 {
        mmio_write(
            se_base,
            match keyslot {
                0 => mmio::security_engine::RSA_KEYTABLE_ACCESS_0,
                1 => mmio::security_engine::RSA_KEYTABLE_ACCESS_1,
                _ => unreachable!(),
            },
            (((perms >> 4) & 4) | (perms & 3)) ^ 7,
        );
    }

    if (perms & 0x80) != 0 {
        mmio_and(
            se_base,
            mmio::security_engine::RSA_SECURITY_PERKEY,
            !(1 << keyslot),
        );
    }
}

pub fn clear_keyslot(se_base: usize, keyslot: u32) {
    if keyslot > 1 {
        loop {}
    }

    for iter in 0..0x40 {
        let value = (keyslot << 7) | iter;
        mmio_write(
            se_base,
            mmio::security_engine::RSA_KEYTABLE_ADDR,
            value | 0x40,
        );
        mmio_write(se_base, mmio::security_engine::RSA_KEYTABLE_DATA, 0);
    }

    for iter in 0..0x40 {
        let value = (keyslot << 7) | iter;
        mmio_write(se_base, mmio::security_engine::RSA_KEYTABLE_ADDR, value);
        mmio_write(se_base, mmio::security_engine::RSA_KEYTABLE_DATA, 0);
    }
}
