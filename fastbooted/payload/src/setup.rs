use core::{arch::naked_asm, panic::PanicInfo};

/// This is useless because of `panic_immediate_abort`
#[panic_handler]
fn panic(_panic_info: &PanicInfo<'_>) -> ! {
    loop {}
}

#[unsafe(link_section = ".text.start")]
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    naked_asm!("B  _init")
}

#[unsafe(link_section = ".text.init")]
pub fn clear_bss() {
    unsafe extern "C" {
        static __bss_start: u8;
        static __bss_end: u8;
    }

    unsafe {
        let start = (&__bss_start) as *const _ as usize;
        let end = (&__bss_end) as *const _ as usize;

        for ptr in start..end {
            *(ptr as *mut u8) = 0;
        }
    }
}

#[unsafe(link_section = ".text.init")]
pub fn relocate() {
    unsafe extern "C" {
        static __text_start_end: u8;
        static __end: u8;
        static __text_start: u8;
    }

    unsafe {
        let unrelocated_start = (&__text_start_end) as *const _ as usize;
        let relocated_end = (&__end) as *const _ as usize;
        let relocated_start = (&__text_start) as *const _ as usize;
        let length = relocated_end - relocated_start;

        if length % 8 != 0 {
            loop {}
        }

        for x in 0..(length / 8) {
            *((relocated_start + (x * 8)) as *mut u64) =
                *((unrelocated_start + (x * 8)) as *mut u64)
        }
    }

    tx2_common::utils::invalidate_unified_and_data_caches();
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.init")]
pub extern "C" fn _init() -> ! {
    clear_bss();
    crate::main();
}
