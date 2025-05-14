use crate::def_multi_reg32;

pub const SE_BASE: usize = 0x3ac0000;

def_multi_reg32!(
    STATUS = 0x2f0,
    AES0_STATUS = 0x2f8,
    CRYPTO_KEYTABLE_ADDR = 0x2bc,
    CRYPTO_KEYTABLE_DATA = 0x2c0
);
