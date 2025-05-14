use crate::mmio::mmio_read;

pub fn read_usec_cntr() -> u32 {
    mmio_read(crate::mmio::tmr::TIMER_USEC_BASE, 0)
}
