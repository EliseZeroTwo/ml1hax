use tx2_common::{mmio, utils::usleep};

use crate::{
    fastboot::{FastbootCommandHandlerRes, fastboot_fail, fastboot_info, fastboot_okay},
    try_something,
};

pub(crate) fn fastboot_owo(_arg: &[u8]) -> FastbootCommandHandlerRes {
    match fastboot_okay(b"Meow from the Bootloader! Good security as ever NVidia!") {
        Ok(_) => FastbootCommandHandlerRes::Continue,
        Err(_) => FastbootCommandHandlerRes::DropDevice,
    }
}

pub(crate) fn fastboot_reboot(_arg: &[u8]) -> FastbootCommandHandlerRes {
    try_something!(fastboot_okay(b"Rebooting!"));
    usleep(25000);
    crate::reboot();
    loop {}
}

pub(crate) fn fastboot_poweroff(_arg: &[u8]) -> FastbootCommandHandlerRes {
    try_something!(fastboot_okay(b"Powering off!"));
    usleep(25000);
    crate::poweroff();
    loop {}
}

pub(crate) fn fastboot_3p_reboot(_arg: &[u8]) -> FastbootCommandHandlerRes {
    try_something!(fastboot_info(b"Reading addr..."));

    let ramdump_state_addr =
        mmio::mmio_read(mmio::scratch::SCRATCH_BASE, mmio::scratch::SECURE_RSV43_0) as u64
            | ((mmio::mmio_read(mmio::scratch::SCRATCH_BASE, mmio::scratch::SECURE_RSV43_1)
                as u64)
                << 32);

    try_something!(fastboot_info(b"RAMDUMP state address:"));
    try_something!(fastboot_info(&payload_helpers::u64_to_bytes(
        ramdump_state_addr
    )));
    if ramdump_state_addr == 0 {
        _ = fastboot_fail(b"RAMDUMP state address is NULL :c");
        return FastbootCommandHandlerRes::DropDevice;
    }

    unsafe { core::ptr::write_volatile(ramdump_state_addr as *mut u32, 0xdeadbeef) };
    try_something!(fastboot_okay(b"Written Rebooting to 3P!"));
    usleep(25000);
    crate::reboot();
    loop {}
}
