use core::str::FromStr;

use gpt_disk_types::{GptPartitionName, GptPartitionType, LbaLe};

use crate::{
    fastboot::{FastbootCommandHandlerRes, fastboot_fail, fastboot_okay, flash::FlashDevice},
    handle_disk_res, try_something,
};

const DTBHAX_BACKUP_PARTITION: &str = "DTBHAX";
const KDTB_PARTITION: &str = "kernel-dtb";
const USER_PARTITION: &str = "UDA";

const DTBHAX_LOAD_ADDR: usize = 0x92000000;
const CBOOT_LOAD_ADDR: usize = 0x96000000;
/// Not accurate but good enough
const CBOOT_END_ADDR: usize = 0x96071000;

const SLIDE_OFFSET: u64 = (CBOOT_LOAD_ADDR - DTBHAX_LOAD_ADDR) as u64;

const NEEDED_LEN: u64 = const {
    let len = (CBOOT_END_ADDR - DTBHAX_LOAD_ADDR) as u64;
    assert!(len % 4096 == 0);
    assert!(len > SLIDE_OFFSET);
    assert!(len > PAYLOAD.len() as u64);
    assert!(len > SLIDE_OFFSET + (PAYLOAD.len() as u64));
    len
};

const SLIDE_LEN: u64 = const {
    let len = (CBOOT_END_ADDR - CBOOT_LOAD_ADDR) as u64;
    assert!(len % 4096 == 0);
    len
};

const PAYLOAD: &[u8] = include_bytes!("dtbhax.bin");

/// Warning: This code works, it gets coldboot execution, but the implementation is slightly broken and will brick the console (it will just get stuck in a loop of calling itself).
/// The Jump slide is incorrect, you NEED to copy the CBoot implementation as it is in memory to here instead, and replace an instruction somewhere in the call chain of where it goes to load the kernel-dtb with the Jump, and then it will work.
/// I will get around to fixing this at some point eventually, if nobody else does so.
pub fn fastboot_dtbhax_setup(_args: &[u8]) -> FastbootCommandHandlerRes {
    try_something!(fastboot_fail(
        b"Stubbed for now, as this code contains a bug, and bricked a console!"
    ));
    return FastbootCommandHandlerRes::Continue;
    // let Some(mut flash) = FlashDevice::new(false) else {
    //     _ = fastboot_fail(b"Failed to open flash block device");
    //     return FastbootCommandHandlerRes::DropDevice;
    // };

    // let mut disk = handle_disk_res!(gpt_disk_io::Disk::new(&mut flash), b"DTBHAX-Open");
    // let mut block_buf = [0u8; 4096 * 2];

    // let mut header = handle_disk_res!(
    //     disk.read_primary_gpt_header(&mut block_buf),
    //     b"DTBHAX-RPGPTH"
    // );
    // let mut entry_count = header.number_of_partition_entries.to_u32();
    // if entry_count != 39 {
    //     _ = fastboot_fail(b"Failed to read partition layout");
    //     return FastbootCommandHandlerRes::DropDevice;
    // }
    // entry_count += 1;
    // header.number_of_partition_entries.set(entry_count);
    // let Ok(layout) = header.get_partition_entry_array_layout() else {
    //     _ = fastboot_fail(b"header.number_of_partition_entries != 39 ???");
    //     return FastbootCommandHandlerRes::DropDevice;
    // };
    // let mut array = handle_disk_res!(
    //     disk.read_gpt_partition_entry_array(layout, &mut block_buf),
    //     b"DTBHAX-GETITER"
    // );
    // let Ok(dtbhax_entry_name) = GptPartitionName::from_str(DTBHAX_BACKUP_PARTITION) else {
    //     _ = fastboot_fail(b"Invalid partition name (dtbhax)?");
    //     return FastbootCommandHandlerRes::DropDevice;
    // };
    // let Ok(kdtb_entry_name) = GptPartitionName::from_str(KDTB_PARTITION) else {
    //     _ = fastboot_fail(b"Invalid partition name (kernel-dtb)?");
    //     return FastbootCommandHandlerRes::DropDevice;
    // };
    // let Ok(user_entry_name) = GptPartitionName::from_str(USER_PARTITION) else {
    //     _ = fastboot_fail(b"Invalid partition name (user)?");
    //     return FastbootCommandHandlerRes::DropDevice;
    // };

    // let mut kdtb_partition_found = false;
    // for idx in 0..=entry_count {
    //     let Some(entry) = array.get_partition_entry_mut(idx) else {
    //         _ = fastboot_fail(b"Missing partition (ktbd out-of-entries)");
    //         return FastbootCommandHandlerRes::DropDevice;
    //     };

    //     if entry.name == kdtb_entry_name {
    //         kdtb_partition_found = true;
    //         entry.name = dtbhax_entry_name;
    //         break;
    //     }
    // }

    // if !kdtb_partition_found {
    //     _ = fastboot_fail(b"Missing partition (kernel-dtb out-of-indexes)");
    //     return FastbootCommandHandlerRes::DropDevice;
    // };

    // let mut new_lbas = None;
    // for idx in 0..=entry_count {
    //     let Some(entry) = array.get_partition_entry_mut(idx) else {
    //         _ = fastboot_fail(b"Missing partition (3)");
    //         return FastbootCommandHandlerRes::DropDevice;
    //     };

    //     if entry.name == user_entry_name {
    //         let needed_lbas = NEEDED_LEN / 4096;
    //         let ending_lba = entry.ending_lba.to_u64();
    //         let starting_lba = ending_lba - needed_lbas;
    //         entry.ending_lba = LbaLe::from_u64(starting_lba - 1);
    //         new_lbas = Some((starting_lba, ending_lba));
    //         break;
    //     }
    // }

    // let Some((new_kdtb_start_lba, new_kdtb_end_lba)) = new_lbas else {
    //     _ = fastboot_fail(b"Missing partition (3)");
    //     return FastbootCommandHandlerRes::DropDevice;
    // };

    // for idx in 0..=entry_count {
    //     let Some(entry) = array.get_partition_entry_mut(idx) else {
    //         break;
    //     };

    //     if entry.is_used() || !entry.name.is_empty() {
    //         continue;
    //     }

    //     entry.name = kdtb_entry_name;
    //     entry.partition_type_guid = GptPartitionType::BASIC_DATA;
    //     entry.starting_lba = LbaLe::from_u64(new_kdtb_start_lba);
    //     entry.ending_lba = LbaLe::from_u64(new_kdtb_end_lba);

    //     let array_crc32 = array.calculate_crc32();
    //     handle_disk_res!(
    //         disk.write_gpt_partition_entry_array(&array),
    //         b"DTBHAX-WRITE-ARRAY"
    //     );
    //     drop(array);

    //     header.partition_entry_array_crc32 = array_crc32;
    //     header.update_header_crc32();

    //     try_something!(
    //         disk.write_primary_gpt_header(&header, &mut block_buf),
    //         b"DTBHAX-WRITE-HDR"
    //     );

    //     drop(disk);
    //     try_something!(
    //         flash.device.write(PAYLOAD, new_kdtb_start_lba * 4096),
    //         b"DTBHAX-WRITE-PAYLOAD"
    //     );
    //     for sector in 0..(SLIDE_LEN / 4096) {
    //         let mut buffer = [0u8; 4096];
    //         for index in 0..(4096 / 4) {
    //             let instruction_index = (sector * (4096 / 4)) + index;
    //             let instruction = 0x1700_0000u32 - instruction_index as u32;
    //             let bidx = index as usize * 4;
    //             buffer[bidx..bidx + 4].copy_from_slice(&instruction.to_le_bytes());
    //         }
    //         try_something!(
    //             flash.device.write(
    //                 &buffer,
    //                 ((new_kdtb_start_lba + sector) * 4096) + SLIDE_OFFSET
    //             ),
    //             b"DTBHAX-WRITE-SLIDE"
    //         );
    //     }

    //     try_something!(fastboot_okay(b"Success!"));
    //     return FastbootCommandHandlerRes::Continue;
    // }

    // _ = fastboot_fail(b"Out of indexes");
    // FastbootCommandHandlerRes::DropDevice
}
