use crate::def_multi_reg32;

pub const SCRATCH_BASE: usize = 0x0C39_0000;

def_multi_reg32!(
    SECURE_RSV43_0 = 0x7a8,
    SECURE_RSV43_1 = 0x7ac,
    SECURE_RSV54_0 = 0x800
);
