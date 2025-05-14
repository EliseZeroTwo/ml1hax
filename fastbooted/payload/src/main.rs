#![no_std]
#![no_main]
#![feature(concat_bytes)]
#![allow(clippy::needless_range_loop)]

use core::arch::asm;

#[cfg(feature = "uspee")]
pub mod fastboot;
mod setup;

fn memsearch<const LEN: usize>(base_addr: u64, top_addr: u64, search: &[u8; LEN]) -> Option<u64> {
    for addr in base_addr..top_addr {
        let x = unsafe { core::slice::from_raw_parts_mut(addr as *mut u8, LEN) };
        if x == search {
            return Some(addr);
        }
    }

    None
}

fn memsearch_instr(base_addr: u64, top_addr: u64, instr: u32, mask: u32) -> Option<u64> {
    let mut search_addr = base_addr;
    while search_addr < top_addr {
        let x = unsafe { core::ptr::read_volatile(search_addr as *mut u32) };
        if (x & mask) == (instr & mask) {
            return Some(search_addr);
        }
        search_addr += 4;
    }

    None
}

fn find_replace<const LEN: usize>(
    base_addr: u64,
    top_addr: u64,
    search: &[u8; LEN],
    replace: &[u8; LEN],
) -> bool {
    let mut found = false;
    for addr in base_addr..top_addr {
        let x = unsafe { core::slice::from_raw_parts_mut(addr as *mut u8, LEN) };
        if x == search {
            x.copy_from_slice(replace);
            tx2_common::utils::clear_and_invalidate_cache_for_address_range(
                addr & 0xFFFF_0000,
                0x1_0000,
            );
            found = true;
        }
    }

    found
}

fn poweroff() {
    unsafe {
        asm!(
            "smc #0",
            in("x0") 0x84000008u64,
            in("x1") 0u64,
            in("x2") 0u64,
            in("x3") 0u64
        )
    };
}

fn reboot() {
    unsafe {
        asm!(
            "smc #0",
            in("x0") 0x84000009u64,
            in("x1") 0u64,
            in("x2") 0u64,
            in("x3") 0u64
        )
    };
}

#[cfg(feature = "dump-memory")]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct Header {
    pub mode: u32,
    pub address: u32,
    pub dump_hi_nibble: u32,
}

#[cfg(feature = "dump-memory")]
#[unsafe(link_section = ".text.header")]
#[unsafe(no_mangle)]
static mut HEADER: Header = Header {
    mode: 0x1010_1010,
    address: 0x6969_6969,
    dump_hi_nibble: 0x7979_7979,
};

// Takes roughly 3060000µs to run at base
#[cfg(feature = "dump-memory")]
fn main() -> ! {
    let header = unsafe { core::ptr::read_volatile(&raw mut HEADER) };

    let byte = unsafe { core::ptr::read_volatile(header.address as usize as *const u8) };

    let nibble = match header.dump_hi_nibble != 0 {
        true => byte >> 4,
        false => byte & 0b1111,
    };

    usleep(100000 * nibble as u32);
    reboot();
    loop {}
}

#[cfg(feature = "dump-instruction-addr")]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct Header {
    pub mode: u32,
    pub start_address: u32,
    pub end_address: u32,
    pub instruction: u32,
    pub mask: u32,
    pub offset: u32,
    pub dump_hi_nibble: u32,
    pub calculate_base_timing: u32,
}

#[cfg(feature = "dump-instruction-addr")]
#[unsafe(link_section = ".text.header")]
#[unsafe(no_mangle)]
static mut HEADER: Header = Header {
    mode: 0x2020_2020,
    start_address: 0x6969_6969,
    end_address: 0x6969_6969,
    instruction: 0x6969_6969,
    mask: 0x6969_6969,
    offset: 0x6969_6969,
    dump_hi_nibble: 0x7979_7979,
    calculate_base_timing: 0x7979_7979,
};

#[cfg(feature = "dump-instruction-addr")]
fn main() -> ! {
    let header = unsafe { core::ptr::read_volatile(&raw mut HEADER) };

    if let Some(addr) = memsearch_instr(
        header.start_address as u64,
        header.end_address as u64,
        header.instruction,
        header.mask, // u32::from_le_bytes([0x38, 0x00, 0xa0, 0x52]), // mov _, #0x10000  <- used in transport_usbf_send
                     // !0b11111,
    ) {
        let byte = (addr >> (header.offset * 8)) as u8;

        let nibble = match header.dump_hi_nibble != 0 {
            true => byte >> 4,
            false => byte & 0b1111,
        };

        if header.calculate_base_timing == 0 {
            usleep(100000 * nibble as u32);
        }
        reboot();
        loop {}
    }

    loop {}
}

#[cfg(feature = "uspee")]
fn main() -> ! {
    // type TransportUsbfOpen = extern "C" fn(u32, *const fastboot::externs::UsbfInfo) -> u32;
    // const TRANSPORT_USBF_OPEN_ADDR: u64 = 0x96059608u64;
    // let transport_usbf_open_inner: TransportUsbfOpen =
    //     unsafe { core::mem::transmute(TRANSPORT_USBF_OPEN_ADDR as *const ()) };

    // let info = fastboot::externs::FASTBOOT_INFO;

    // loop {
    //     transport_usbf_open_inner(0, &raw const info);
    // }

    // use fastboot::run_fastboot_server;

    fastboot::run_fastboot_server();
}

// fn main() -> ! {
//     // const MAGIC: u32 = 0x6969_6969;
//     // const PMC_SCRATCH_BASE: u64 = 0x0c390000;
//     // const S0_ADDRESS: u64 = 0xF000_0000;// + PMC_SCRATCH_BASE;
//     // const S1_ADDRESS: u64 = S0_ADDRESS + 4;

//     // tx1_a57_common::utils::

//     // if unsafe { core::ptr::read_volatile(S0_ADDRESS as *mut u32) }  == MAGIC {// && unsafe { core::ptr::read_volatile(S1_ADDRESS as *mut u32) }  == MAGIC {

//     // } else {
//     //     unsafe {
//     //         core::ptr::write_volatile(S0_ADDRESS as *mut u32, MAGIC);
//     //         core::ptr::write_volatile(S1_ADDRESS as *mut u32, MAGIC);
//     //     }
//     //     reboot();
//     // }

//     loop {}
//     // loop {}
//     // tx1_a57_common::utils::reboot_to_rcm(pmc_base, flow_controller_base);

//     // let pmc_base = 0x0c360000;
//     // tx2_common::mmio::mmio_write(pmc_base, tx1_a57_common::mmio::pmc::APBDEV_PMC_SCRATCH0, 1 << 1);

//     // tx2_common::hardware::usb::disable_endpoint(tx2_common::hardware::usb::EndpointType::Endpoint1Out);
//     // tx2_common::hardware::usb::disable_endpoint(tx2_common::hardware::usb::EndpointType::Endpoint1In);

//     // let mut device = tx2_common::hardware::usb::UsbDeviceContext::new();
//     // device.init();
//     // tx1_a57_common::mmio::mmio_or(
//     //     pmc_base,
//     //     tx1_a57_common::mmio::pmc::APBDEV_PMC_CNTRL,
//     //     tx1_a57_common::mmio::pmc::PMC_CNTRL_MAIN_RST,
//     // );

//     // libusbf::usbf_close(0);s
//     // libusbf::usbf_close(1);
//     // libusbf::usbf_close(2);
//     // libusbf::usbf_close(3);
//     // libusbf::usbf_open(false, libusbf::UsbfUsbClass::Fastboot);
//     // let mut bytes = 0u32;
//     // usbf_send(b"MEOW", &mut bytes, 0x1FFF_FFFF);
//     // usbf_send(b"MEOW", &mut bytes, 0x1FFF_FFFF);
//     // usbf_send(b"MEOW", &mut bytes, 0x1FFF_FFFF);

//     // loop {}

//     // _ = tx2_common::hardware::i2c::init(tx2_common::mmio::i2c::I2C2_BASE);

//     //     for pwm in 0..=255u8 {
//     //         for d in [D::D1, D::D2, D::D3, D::D4, D::D5, D::D6, D::D7, D::D8, D::D9] {
//     //             lp55231.set_pwm(d, pwm);
//     //         }
//     //     }
//     // }

//     // loop {}

//     // if !find_replace(
//     //     0x9600_0258u64,
//     //     0xA800_0000u64,
//     //     b"F\x00a\x00s\x00t\x00b\x00o\x00o\x00t\x00",
//     //     b"M\x00e\x00o\x00w\x00b\x00o\x00o\x00t\x00",
//     // ) {
//     //     loop {}
//     // }

//     // if !find_replace(
//     //     0x9600_0258u64,
//     //     0xB000_0000u64,
//     //     b"Unknown var!",
//     //     b"Meow UwU OwO",
//     // ) {
//     //     loop {}
//     // }

//     // if !find_replace(
//     //     0x9600_0258u64,
//     //     0x9600_0000u64 + 0x200000,
//     //     b"nknown command",
//     //     b"Meow UwU! OwO!",
//     // ) {
//     //     loop {}
//     // }

//     // let last_addr = tx2_common::mmio::mmio_write(base, reg, value);

//     // tx2_common::utils::usleep(1000000);

//     // memsearch(0x9600_0258u64, 0x9600_0000u64 + 0x200000, b"ERROR: Add to request queue failed");
//     // if let Some(mov) = memsearch_instr(
//     //     0x9600_0000u64,
//     //     0x9600_0000u64 + 0x80000,
//     //     u32::from_le_bytes([0x38, 0x00, 0xa0, 0x52]),
//     //     // u32::from_le_bytes([0x3f, 0x00, 0x10, 0x71]),
//     //     !0b11111
//     // ) {
//     //     if let Some(cmp_1) = memsearch_instr(
//     //         0x9600_0000u64,
//     //         0x9600_0000u64 + 0x80000,
//     //         // u32::from_le_bytes([0x38, 0x00, 0xa0, 0x52]),
//     //         u32::from_le_bytes([0x3f, 0x00, 0x10, 0x71]),
//     //         !0b11111
//     //     ) {
//     //         if ((mov - cmp_1) <= 0x1000) || ((cmp_1 - mov) <= 0x1000) {
//     //             poweroff();
//     //         } else if let Some(cmp_2) = memsearch_instr(
//     //             cmp_1 + 4,
//     //             0x9600_0000u64 + 0x80000,
//     //             // u32::from_le_bytes([0x38, 0x00, 0xa0, 0x52]),
//     //             u32::from_le_bytes([0x3f, 0x00, 0x10, 0x71]),
//     //             !0b11111
//     //         ) {
//     //             if ((mov - cmp_2) <= 0x100) || ((cmp_2 - mov) <= 0x100) {
//     //                 reboot();
//     //             }
//     //         }
//     //     }
//     // }

//     // const BL_MASK: u32 = 0b10010100000000000000000000000000;

//     // let mut search_base = 0x9600_0000u64;
//     // while search_base < 0x9600_0000u64 + 0x80000 {
//     //     if let Some(addr) = memsearch_instr(
//     //         search_base,
//     //         0x9600_0000u64 + 0x80000,
//     //         // u32::from_le_bytes([0x00, 0x57, 0xBE, 0x12]),
//     //         // u32::from_le_bytes([0x40, 0xa9, 0xa1, 0xd2]),
//     //         // u32::from_le_bytes([0x38, 0x00, 0xa0, 0x52]), // mov _, #0x10000  <- used in transport_usbf_send
//     //         // u32::from_le_bytes([0x3f, 0x00, 0x10, 0x71]),
//     //         u32::from_le_bytes([0x01, 0x80, 0x80, 0xd2]),
//     //         !0,
//     //         // !0b11111,
//     //     ) {
//     //         search_base = addr;
//     //         // if let Some(addr2) = memsearch_instr(addr - 0x100, addr, u32::from_le_bytes([0x20, 0x00, 0x80, 0x52]), !0b11111) {

//     //         if let Some(addr) = memsearch_instr(addr, addr + 0x10, u32::from_le_bytes([0xdb, 0xeb, 0xff, 0x97]), BL_MASK) {
//     //             poweroff();
//     //         }
//     //     }
//     // }
//     // loop{}
//     // if let Some(addr) = memsearch_instr(
//     //     0x9600_0000u64,
//     //     0x9600_0000u64 + 0x80000,
//     //     // u32::from_le_bytes([0x00, 0x57, 0xBE, 0x12]),
//     //     // u32::from_le_bytes([0x40, 0xa9, 0xa1, 0xd2]),
//     //     u32::from_le_bytes([0x38, 0x00, 0xa0, 0x52]), // mov _, #0x10000  <- used in transport_usbf_send
//     //     // u32::from_le_bytes([0x3f, 0x00, 0x10, 0x71]),
//     //     // u32::from_le_bytes([0x01, 0x80, 0x80, 0xd2]),
//     //     !0b11111,
//     // ) {
//     //     // if let Some(addr2) = memsearch_instr(addr - 0x40, addr, u32::from_le_bytes([0x20, 0x00, 0x80, 0x52]), !0b11111) {
//     //     //     poweroff();
//     //     // } else {
//     //     //     loop {}
//     //     // }
//     //     const MOV_FROM_SP: u32 = u32::from_le_bytes([0xfd, 0x03, 0x00, 0x91]);
//     //     let final_addr = addr - 0x80;
//     //     let mut search_addr = addr - 4;

//     //     while search_addr > final_addr {
//     //         if (unsafe {*(search_addr as *mut u32)} & !0b11111) == (MOV_FROM_SP & !0b11111) {
//     //             // const STP: u32 = u32::from_le_bytes([0xfd, 0x7b, 0xbb, 0xa9]);
//     //             // const STP_MASK: u32 = 0b1111111111 << 10;
//     //             // if (unsafe {*((search_addr - 4) as *mut u32)} & STP_MASK) == STP & STP_MASK {
//     //             // let ptr = (search_addr + 4) as *const ();
//     //             // let function: TransportUsbfSend = unsafe { core::mem::transmute(ptr) };
//     //             const UWU: &[u8] = b"OKAYMeow!";
//     //             let mut x = 0u32;
//     //             // function(UWU.as_ptr(), UWU.len() as _, &mut x as *mut u32, 0xFFFF_FFFF);
//     //             // function(UWU.as_ptr(), UWU.len() as _, &mut x as *mut u32, 0xFFFF_FFFF);
//     //             if let Some(addr) = memsearch_instr(addr, addr + 0x60, u32::from_le_bytes([0x89, 0x08, 0x00, 0x94]), BL_MASK) {
//     //                 let func_addr = ((unsafe { *(addr as *mut u32) } & !0b11111100000000000000000000000000) as u64 + (addr + 4)) as *const ();
//     //                 let function: UsbfTransmit = unsafe { core::mem::transmute(func_addr) };

//     //                 // function(UWU.as_ptr(), UWU.len() as u32, &mut x as *mut u32);

//     //                 poweroff();
//     //             }
//     //             // }
//     //         }

//     //         search_addr -= 4;
//     //     }
//     // }

//     // // if let Some(meow) = memsearch(0x9600_0258u64 + 0x80000, 0x9600_0000u64 + 0xa0000, b"ERROR: Add to request queue failed") {
//     // //     if let Some(meow2) = memsearch(0x9600_0258u64, 0x9600_0000u64 + 0x80000, &meow.to_le_bytes()) {
//     // //         loop {}
//     // //     } else {
//     // //         unsafe {
//     // //             asm!(
//     // //                 "smc #0",
//     // //                 in("x0") 0x84000008u64,
//     // //                 in("x1") 0u64,
//     // //                 in("x2") 0u64,
//     // //                 in("x3") 0u64
//     // //             )
//     // //         };
//     // //     }
//     // // }

//     // // #[expect(deref_nullptr)]
//     // // unsafe { *null_mut() = 123456789 };

//     // // unsafe {
//     // //     asm!(
//     // //         "MOV LR, {}",
//     // //         "RET",
//     // //         in(reg) 0x96000658u64
//     // //     )
//     // // }

//     // loop {}
// }
