#[macro_export]
macro_rules! read_spr {
    ($spr:literal) => {
        {
            let out_val: u64;
            unsafe { core::arch::asm!(concat!("MRS {0}, ", $spr), out(reg) out_val); }
            out_val
        }
    };
}

#[macro_export]
macro_rules! write_spr {
    ($spr:literal, $val:expr) => {
        {
            let val: u64 = $val;
            unsafe { core::arch::asm!(concat!("MSR ", $spr, ", {0}"), in(reg) val); }
        }
    };
}

#[macro_export]
macro_rules! isb {
    () => {
        unsafe {
            core::arch::asm!("isb");
        }
    };
}

#[macro_export]
macro_rules! dc_isw {
    ($val:expr) => {
        {
            let val: u64 = $val;
            unsafe { core::arch::asm!("dc isw, {0}", in(reg) val); }
        }
    };
}

#[macro_export]
macro_rules! dsb_ish {
    () => {{
        unsafe {
            core::arch::asm!("dsb ish");
        }
    }};
}

#[macro_export]
macro_rules! dc_civac {
    ($addr:expr) => {{
        unsafe {
            core::arch::asm!("dc civac, {0}", in(reg) $addr);
        }
    }};
}

#[macro_export]
macro_rules! ic_iallu {
    () => {{
        unsafe {
            core::arch::asm!("ic iallu");
        }
    }};
}
