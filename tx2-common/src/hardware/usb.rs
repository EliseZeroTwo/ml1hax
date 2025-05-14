use modular_bitfield::{
    bitfield,
    prelude::{B1, B10, B11, B15, B16, B17, B2, B20, B22, B24, B28, B3, B4, B5, B6, B7, B8, B9},
};

use crate::{
    mmio::{self, mmio_and, mmio_or, mmio_read, mmio_write, usb::{STATUS_IP, XUSB_DEV_BASE}},
    utils::usleep,
};

#[bitfield(bits = 128)]
pub struct EventTrb {
    #[skip]
    __: u32,
    #[skip]
    __: u32,
    #[skip]
    __: B24,
    pub comp_code: B8,
    pub cycle: B1,
    #[skip]
    __: B9,
    pub trb_type: B6,
    pub emp_id: B5,
    #[skip]
    __: B11,
}

#[bitfield(bits = 128)]
pub struct SetupEventTrb {
    pub data0: u32,
    pub data1: u32,
    pub control_seq_number: B16,
    #[skip]
    __: B8,
    pub comp_code: B8,
    pub cycle: B1,
    #[skip]
    __: B9,
    pub trb_type: B6,
    pub emp_id: B5,
    #[skip]
    __: B11,
}

#[bitfield(bits = 128)]
pub struct DataTrb {
    pub data_buf_ptr_lo: u32,
    pub data_buf_ptr_hi: u32,
    pub trb_tx_len: B17,
    pub td_size: B5,
    pub int_target: B10,
    pub cycle: B1,
    pub evaluate_next_trb: B1,
    pub interrupt_on_short_packet: B1,
    pub no_snoop: B1,
    pub chain_bit: B1,
    pub interrupt_on_completion: B1,
    #[skip]
    __: B4,
    pub trb_type: B6,
    pub dir: B1,
    #[skip]
    __: B15,
}

#[bitfield(bits = 128)]
pub struct NormalTrb {
    pub data_buf_ptr_lo: u32,
    pub data_buf_ptr_hi: u32,
    pub trb_tx_len: B17,
    pub td_size: B5,
    pub int_target: B10,
    pub cycle: B1,
    pub evaluate_next_trb: B1,
    pub interrupt_on_short_packet: B1,
    pub no_snoop: B1,
    pub chain_bit: B1,
    pub interrupt_on_completion: B1,
    pub immediate_data: B1,
    #[skip]
    __: B2,
    pub block_event_interrupt: B1,
    pub trb_type: B6,
    #[skip]
    __: B16,
}

#[bitfield(bits = 128)]
pub struct TransferEventTrb {
    pub data_buf_ptr_lo: u32,
    pub data_buf_ptr_hi: u32,
    pub trb_tx_len: B24,
    pub comp_code: B8,
    pub cycle: B1,
    #[skip]
    __: B1,
    pub event_data: B1,
    #[skip]
    __: B7,
    pub trb_type: B6,
    pub endpoint_id: B5,
    #[skip]
    __: B11,
}

#[bitfield(bits = 128)]
pub struct LinkTrb {
    #[skip]
    __: B4,
    pub ring_seg_ptr_lo: B28,
    pub ring_seg_ptr_hi: u32,
    #[skip]
    __: B22,
    pub int_target: B10,
    pub cycle: B1,
    pub tc: B1,
    #[skip]
    __: B2,
    pub chain: B1,
    pub interrupt_on_completion: B1,
    #[skip]
    __: B4,
    pub trb_type: B6,
    #[skip]
    __: B16,
}

#[bitfield(bits = 512)]
pub struct UsbEndpoint {
    // DWORD0
    pub endpoint_state: B3,
    #[skip]
    __: B5,
    pub mult: B2,
    pub max_pstreams: B5,
    pub lsa: B1,
    pub interval: B8,
    #[skip]
    __: B8,

    // DWORD1
    #[skip]
    __: B1,
    pub cerr: B2,
    pub endpoint_type: B3,
    #[skip]
    __: B1,
    pub hid: B1,
    pub max_burst_size: B8,
    pub max_packet_size: B16,

    // DWORD2
    pub dcs: B1,
    #[skip]
    __: B3,
    pub trd_dequeue_ptr_lo: B28,

    // DWORD3
    pub trd_dequeue_ptr_hi: u32,

    // DWORD4
    pub avg_trb_len: B16,
    pub max_esit_payload: B16,

    // START OF NVIDIA SPECIFIC DEFINITIONS

    // DWORD5
    pub event_data_txlen_acc: B24,
    pub rsvddw5_0: B1,
    pub ptd: B1,
    pub sxs: B1,
    pub seq_num: B5,

    // DWORD6
    pub cprog: B8,
    pub sbyte: B7,
    pub tp: B2,
    pub rec: B1,
    pub cec: B2,
    pub ced: B1,
    pub hsp1: B1,
    pub rty1: B1,
    pub std: B1,
    pub status: B8,

    // DWORD7
    pub data_offset: B17,
    pub rsvddw6_0: B4,
    pub lpa: B1,
    pub num_trb: B5,
    pub num_p: B5,

    // DWORD8
    pub scratch_pad0: u32,

    // DWORD9
    pub scratch_pad1: u32,

    // DWORD10
    pub cping: B8,
    pub sping: B8,
    pub tc: B2,
    pub ns: B1,
    pub ro: B1,
    pub tlm: B1,
    pub dlm: B1,
    pub hsp2: B1,
    pub rty2: B1,
    pub stop_rec_req: B8,

    // DWORD11
    pub device_addr: B8,
    pub hub_addr: B8,
    pub root_port_num: B8,
    pub slot_id: B8,

    // DWORD12
    pub routing_string: B20,
    pub speed: B4,
    pub lpu: B1,
    pub mtt: B1,
    pub hub: B1,
    pub dci: B5,

    // DWORD13
    pub tthub_slot_id: B8,
    pub ttport_num: B8,
    pub ssf: B4,
    pub sps: B2,
    pub int_target: B10,

    // DWORD14
    pub frz: B1,
    pub end: B1,
    pub elm: B1,
    pub mrx: B1,
    pub ep_linklo: B28,

    // DWORD15
    pub ep_linkhi: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum EndpointType {
    Endpoint0In,
    Endpoint0Out,
    Endpoint1Out,
    Endpoint1In,
}

pub fn disable_endpoint(endpoint: EndpointType) -> bool {
    // Endpoint 0 cannot be disabled
    if endpoint == EndpointType::Endpoint0In || endpoint == EndpointType::Endpoint0Out {
        return false;
    }

    mmio_or(
        XUSB_DEV_BASE,
        mmio::usb::DEV_XHCI_ENDPOINT_RELOAD,
        1 << endpoint as u32,
    );
    poll_field(
        XUSB_DEV_BASE,
        mmio::usb::DEV_XHCI_ENDPOINT_RELOAD,
        1 << endpoint as u32,
        0,
        1000,
    )
}

fn poll_field(
    base: usize,
    reg: usize,
    mask: u32,
    expected_value: u32,
    mut timeout_us: u32,
) -> bool {
    while timeout_us > 0 {
        let data = mmio_read(base, reg);

        if data & mask == expected_value {
            return true;
        }

        usleep(1);
        timeout_us -= 1;
    }

    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbSpeed {
    Full,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbDeviceState {
    Default,
    Connected,
    Disconnected,
    Reset,
    AddressedStatusPending,
    Addressed,
    ConfiguredStatusPending,
    Configured,
    Suspended,
}

pub struct UsbDeviceContext {
    pub device_state: UsbDeviceState,
    pub port_speed: UsbSpeed,
    pub initialized: u32,
    pub enumerated: u32,
    pub bytes_txfred: u32,
    pub tx_count: u32,
    pub cntrl_seq_num: u32,
    pub setup_pkt_index: u32,
    pub config_num: u32,
    pub interface_num: u32,
    pub wait_for_event: u32,

    pub control_endpoint_enqueue_ptr: *mut DataRing,
    pub control_endpoint_dequeue_ptr: *mut DataRing,
    pub control_producer_cycle_state: u32,

    pub bulk_out_endpoint_enqueue_ptr: *mut NormalTrb,
    pub bulk_out_endpoint_dequeue_ptr: *mut NormalTrb,
    pub bulk_out_producer_cycle_state: u32,

    pub bulk_in_endpoint_enqueue_ptr: *mut NormalTrb,
    pub bulk_in_endpoint_dequeue_ptr: *mut NormalTrb,
    pub bulk_in_producer_cycle_state: u32,

    pub event_enqueue_ptr: *mut EventTrb,
    pub event_dequeue_ptr: *mut EventTrb,
    pub event_ccs: bool,

    pub dma_er_start_address: *mut EventTrb,
    pub dma_endpoint_context_start_addr: *mut UsbEndpoint,
}

impl UsbDeviceContext {
    pub const fn new() -> Self {
        Self {
            device_state: UsbDeviceState::Default,
            port_speed: UsbSpeed::High,
            initialized: 0,
            enumerated: 0,
            bytes_txfred: 0,
            tx_count: 0,
            cntrl_seq_num: 0,
            setup_pkt_index: 0,
            config_num: 0,
            interface_num: 0,
            wait_for_event: 0,
            control_endpoint_enqueue_ptr: core::ptr::null_mut(),
            control_endpoint_dequeue_ptr: core::ptr::null_mut(),
            control_producer_cycle_state: 0,
            bulk_out_endpoint_enqueue_ptr: core::ptr::null_mut(),
            bulk_out_endpoint_dequeue_ptr: core::ptr::null_mut(),
            bulk_out_producer_cycle_state: 0,
            bulk_in_endpoint_enqueue_ptr: core::ptr::null_mut(),
            bulk_in_endpoint_dequeue_ptr: core::ptr::null_mut(),
            bulk_in_producer_cycle_state: 0,
            event_enqueue_ptr: core::ptr::null_mut(),
            event_dequeue_ptr: core::ptr::null_mut(),
            event_ccs: false,
            dma_er_start_address: core::ptr::null_mut(),
            dma_endpoint_context_start_addr: core::ptr::null_mut(),
        }
    }
}

impl UsbDeviceContext {
    pub fn init(&mut self) {
        self.device_state = UsbDeviceState::Configured;
        self.port_speed = UsbSpeed::High;

        // Set EVENT_RING_HALT
        mmio_or(XUSB_DEV_BASE, mmio::usb::DEV_XHCI_CONTROL, mmio::usb::EVENT_RING_HALT);
        
        // Init event rings
        self.init_endpoint_event_ring();

        // Clear EVENT_RING_HALT
        mmio_and(XUSB_DEV_BASE, mmio::usb::DEV_XHCI_CONTROL, !mmio::usb::EVENT_RING_HALT);

        mmio_write(XUSB_DEV_BASE, mmio::usb::DEV_XHCI_ENDPOINT_PAUSE, 0xd);
        while mmio_read(XUSB_DEV_BASE, mmio::usb::DEV_XHCI_ENDPOINT_PAUSE) != 0xd {}

        // Initialise EP0
        self.init_endpoint(EndpointType::Endpoint0In);
        self.init_endpoint(EndpointType::Endpoint1In);
        self.init_endpoint(EndpointType::Endpoint1Out);

        mmio_write(XUSB_DEV_BASE, mmio::usb::DEV_XHCI_ECP_LO, self.dma_endpoint_context_start_addr as u32);
        mmio_write(XUSB_DEV_BASE, mmio::usb::DEV_XHCI_ECP_HI, ((self.dma_endpoint_context_start_addr as usize) >> 32) as u32);
    }

    fn init_endpoint_event_ring(&mut self) {
        unsafe { EVENT_RINGS = core::mem::zeroed() };
    
        self.event_dequeue_ptr = &raw mut EVENT_RINGS as *mut _;
        self.event_enqueue_ptr = self.event_dequeue_ptr;
        self.event_ccs = true;
    
        // Configure event ring segment 0
        let dma_buf = self.event_dequeue_ptr;

        self.dma_er_start_address = dma_buf;

        mmio_write(XUSB_DEV_BASE, mmio::usb::DEV_XHCI_EVENT_RING_SEG0_BASE_ADDR_LO, dma_buf as u32);
        mmio_write(XUSB_DEV_BASE, mmio::usb::DEV_XHCI_EVENT_RING_SEG0_BASE_ADDR_HI, ((dma_buf as usize) >> 32) as u32);
    
        // Configure event ring segment 1
        let tbuf = unsafe { &raw mut EVENT_RINGS[1] };
        mmio_write(XUSB_DEV_BASE, mmio::usb::DEV_XHCI_EVENT_RING_SEG1_BASE_ADDR_LO, tbuf as u32);
        mmio_write(XUSB_DEV_BASE, mmio::usb::DEV_XHCI_EVENT_RING_SEG1_BASE_ADDR_HI, ((tbuf as usize) >> 32) as u32);
    
        // Configure event ring segment sizes
        mmio_write(XUSB_DEV_BASE, mmio::usb::DEV_XHCI_EVENT_RING_TABLE_SIZE, (ITEMS_PER_RING as u32) << 16 | ITEMS_PER_RING as u32);
    
        // Set Enqueue/Producer Cycle State for controller
        let mut erep_lo = mmio_read(XUSB_DEV_BASE, mmio::usb::DEV_XHCI_EVENT_RING_ENQUEUE_POINTER_LO);
        erep_lo |= self.event_ccs as u32;
        erep_lo &= !(1 << 1);
        erep_lo &= 0b1111;
        erep_lo |= dma_buf as u32 & !0b1111;
        mmio_write(XUSB_DEV_BASE, mmio::usb::DEV_XHCI_EVENT_RING_ENQUEUE_POINTER_LO, erep_lo);
        mmio_write(XUSB_DEV_BASE, mmio::usb::DEV_XHCI_EVENT_RING_ENQUEUE_POINTER_HI, ((dma_buf as usize) >> 32) as u32);
    
        // Set the Dequeue pointer
        let mut erdp_lo = mmio_read(XUSB_DEV_BASE, mmio::usb::DEV_XHCI_EVENT_RING_ENQUEUE_POINTER_LO);
        erdp_lo &= 0b1111;
        erdp_lo |= dma_buf as u32 & !0b1111;
        mmio_write(XUSB_DEV_BASE, mmio::usb::DEV_XHCI_EVENT_RING_DEQUEUE_POINTER_LO, erdp_lo);
        mmio_write(XUSB_DEV_BASE, mmio::usb::DEV_XHCI_EVENT_RING_DEQUEUE_POINTER_HI, 0);
    }

    fn init_endpoint_context(&mut self, mut endpoint: EndpointType) {
        if endpoint == EndpointType::Endpoint0Out {
            endpoint = EndpointType::Endpoint0In;
        }
    
        let endpoint_info = unsafe { &mut ENDPOINTS.0[endpoint as usize] };
    
        *endpoint_info = unsafe { core::mem::zeroed() };
    
        match endpoint {
            EndpointType::Endpoint0In
            | EndpointType::Endpoint0Out => {
                endpoint_info.set_endpoint_state(1);
                endpoint_info.set_cerr(3);
                endpoint_info.set_max_burst_size(0);
                endpoint_info.set_max_packet_size(64);
                endpoint_info.set_dcs(1);
                endpoint_info.set_cec(3);

                self.control_producer_cycle_state = 1;
                self.control_endpoint_dequeue_ptr = &raw mut EP0_RING;
                self.control_endpoint_enqueue_ptr = &raw mut EP0_RING;

                endpoint_info.set_avg_trb_len(8);
                endpoint_info.set_endpoint_type(4);

                let dma_buf = &raw mut EP0_RING;
                endpoint_info.set_trd_dequeue_ptr_lo((dma_buf as u32) >> 4);
                endpoint_info.set_trd_dequeue_ptr_hi(((dma_buf as usize) >> 32) as u32);

                let link_trb = unsafe { &mut *((&raw mut EP0_RING).add(ITEMS_PER_RING - 1) as *mut LinkTrb) };
                link_trb.set_tc(1);
                link_trb.set_ring_seg_ptr_lo((dma_buf as u32) >> 4);
                link_trb.set_ring_seg_ptr_hi(((dma_buf as usize) >> 32) as u32);
                link_trb.set_trb_type(6);
            },
            EndpointType::Endpoint1Out => {
                endpoint_info.set_endpoint_state(1);
                endpoint_info.set_cerr(3);
                endpoint_info.set_max_burst_size(0);
                endpoint_info.set_dcs(1);
                endpoint_info.set_cec(3);
                
                self.bulk_out_producer_cycle_state = 1;
                self.bulk_out_endpoint_dequeue_ptr = &raw mut EP1_OUT_RING as *mut _;
                self.bulk_out_endpoint_enqueue_ptr = &raw mut EP1_OUT_RING as *mut _;

                match self.port_speed {
                    UsbSpeed::Full => {
                        endpoint_info.set_avg_trb_len(512);
                        endpoint_info.set_max_packet_size(512);
                    },
                    UsbSpeed::High => {
                        endpoint_info.set_avg_trb_len(512);
                        endpoint_info.set_max_packet_size(64);
                    },
                }

                endpoint_info.set_endpoint_type(2);
                
                let dma_buf = &raw mut EP1_OUT_RING;
                endpoint_info.set_trd_dequeue_ptr_lo((dma_buf as u32) >> 4);
                endpoint_info.set_trd_dequeue_ptr_hi(((dma_buf as usize) >> 32) as u32);

                let link_trb = unsafe { &mut *((&raw mut EP1_OUT_RING).add(ITEMS_PER_RING - 1) as *mut LinkTrb) };
                link_trb.set_tc(1);
                link_trb.set_ring_seg_ptr_lo((dma_buf as u32) >> 4);
                link_trb.set_ring_seg_ptr_hi(((dma_buf as usize) >> 32) as u32);
                link_trb.set_trb_type(6);
            },
            EndpointType::Endpoint1In => {
                endpoint_info.set_endpoint_state(1);
                endpoint_info.set_cerr(3);
                endpoint_info.set_max_burst_size(0);
                endpoint_info.set_dcs(1);
                endpoint_info.set_cec(3);

                self.bulk_in_producer_cycle_state = 1;
                self.bulk_in_endpoint_dequeue_ptr = &raw mut EP1_IN_RING as *mut _;
                self.bulk_in_endpoint_enqueue_ptr = &raw mut EP1_IN_RING as *mut _;

                endpoint_info.set_endpoint_type(6);
                
                let dma_buf = &raw mut EP1_IN_RING;
                endpoint_info.set_trd_dequeue_ptr_lo((dma_buf as u32) >> 4);
                endpoint_info.set_trd_dequeue_ptr_hi(((dma_buf as usize) >> 32) as u32);
                
                match self.port_speed {
                    UsbSpeed::Full => {
                        endpoint_info.set_avg_trb_len(512);
                        endpoint_info.set_max_packet_size(512);
                    },
                    UsbSpeed::High => {
                        endpoint_info.set_avg_trb_len(512);
                        endpoint_info.set_max_packet_size(64);
                    },
                }

                let link_trb = unsafe { &mut *((&raw mut EP1_IN_RING).add(ITEMS_PER_RING - 1) as *mut LinkTrb) };
                link_trb.set_tc(1);
                link_trb.set_ring_seg_ptr_lo((dma_buf as u32) >> 4);
                link_trb.set_ring_seg_ptr_hi(((dma_buf as usize) >> 32) as u32);
                link_trb.set_trb_type(6);
            },
        }

        if endpoint == EndpointType::Endpoint0In {
            self.dma_endpoint_context_start_addr = endpoint_info as *mut _;
        }
    }

    fn init_endpoint(&mut self, endpoint: EndpointType) {
        init_transfer_ring(endpoint);

        self.init_endpoint_context(EndpointType::Endpoint0In);

        mmio_and(XUSB_DEV_BASE, mmio::usb::DEV_XHCI_ENDPOINT_PAUSE, !(1 << endpoint as u32));
        mmio_and(XUSB_DEV_BASE, mmio::usb::DEV_XHCI_ENDPOINT_HALT_DCI, !(1 << endpoint as u32));
    }

    pub fn poll_for_event(&mut self, timeout_us: u32) -> bool {
        if !poll_field(XUSB_DEV_BASE, mmio::usb::DEV_XHCI_STATUS, 1 << STATUS_IP, 1 << STATUS_IP, timeout_us) {
            return false;
        }

        mmio_or(XUSB_DEV_BASE, mmio::usb::DEV_XHCI_STATUS, 1 << STATUS_IP);

        let dma_addr = ((mmio_read(XUSB_DEV_BASE, mmio::usb::DEV_XHCI_EVENT_RING_ENQUEUE_POINTER_LO) as usize) | ((mmio_read(XUSB_DEV_BASE, mmio::usb::DEV_XHCI_EVENT_RING_ENQUEUE_POINTER_HI) as usize) << 32)) & !(0b1111);

        let trb_index = (dma_addr - self.dma_er_start_address as usize) / core::mem::size_of::<EventTrb>();

        self.event_enqueue_ptr = unsafe { (&raw mut EVENT_RINGS as *mut EventTrb).add(trb_index) } ;

        true
    }
}

const ITEMS_PER_RING: usize = 16;

#[repr(C, align(16))]
pub struct DataRing(pub [DataTrb; ITEMS_PER_RING]);

#[repr(C, align(16))]
pub struct EventRing(pub [EventTrb; ITEMS_PER_RING]);

static mut EVENT_RINGS: [EventRing; 2] = unsafe { core::mem::zeroed() };
static mut EP0_RING: DataRing = unsafe { core::mem::zeroed() };
static mut EP1_OUT_RING: DataRing = unsafe { core::mem::zeroed() };
static mut EP1_IN_RING: DataRing = unsafe { core::mem::zeroed() };

#[repr(C, align(16))]
pub struct EndpointContexts(pub [UsbEndpoint; 4]);

static mut ENDPOINTS: EndpointContexts = unsafe { core::mem::zeroed() };

fn init_transfer_ring(endpoint: EndpointType) {
    match endpoint {
        EndpointType::Endpoint0In
        | EndpointType::Endpoint0Out => {
            unsafe { EP0_RING = core::mem::zeroed(); }
        },
        EndpointType::Endpoint1Out => {
            unsafe { EP1_OUT_RING = core::mem::zeroed(); }
        },
        EndpointType::Endpoint1In => {
            unsafe { EP1_IN_RING = core::mem::zeroed(); }
        },
    }
}

// pub struct XUsbDeviceContext {
//     pub control_trb_endpoint0_enqueue_ptr: *mut core::ffi::c_void,
//     pub control_trb_endpoint0_dequeue_ptr: *mut core::ffi::c_void,
//     pub control_producer_cycle_state: u32,
// }
