pub mod i2c;
pub mod scratch;
pub mod security_engine;
pub mod tmr;
pub mod usb;

#[macro_export]
macro_rules! def_multi_reg32 {
    ($($name:ident = $offset:expr),*) => {
        $(
            pub const $name: usize = $offset;
        )*
    };
}

/// Ensure base is passed correctly, there are *zero* safety guarantees
#[inline(always)]
pub fn mmio_read(base: usize, reg: usize) -> u32 {
    unsafe { core::ptr::read_volatile((base + reg) as *const u32) }
}

/// Ensure base is passed correctly, there are *zero* safety guarantees
#[inline(always)]
pub fn mmio_write(base: usize, reg: usize, value: u32) {
    unsafe {
        core::ptr::write_volatile((base + reg) as *mut u32, value);
    }
}

/// Ensure base is passed correctly, there are *zero* safety guarantees
#[inline(always)]
pub fn mmio_or(base: usize, reg: usize, value: u32) {
    mmio_write(base, reg, mmio_read(base, reg) | value);
}

/// Ensure base is passed correctly, there are *zero* safety guarantees
#[inline(always)]
pub fn mmio_and(base: usize, reg: usize, mask: u32) {
    mmio_write(base, reg, mmio_read(base, reg) & mask);
}
