#![feature(is_some_with)]
use std::{
    env,
    fs::File,
    io::Write,
    mem::transmute,
    ptr::{addr_of, null_mut},
};

use winapi::um::sysinfoapi::GetSystemFirmwareTable;

#[derive(Debug)]
#[repr(C)]
pub struct Smbios
{
    pub calling_method: u8,
    pub major_version:  u8,
    pub minor_version:  u8,
    pub dmi_revision:   u8,
    pub length:         u32,
    pub table_data:     *mut u8,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct SmbiosHeader
{
    pub header_type: SmbiosHeaderType,
    pub length:      u8,
    pub handle:      u16,
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum SmbiosHeaderType
{
    Bios               = 0,
    SysInfo            = 1,
    BoardInfo          = 2,
    SystemEnclosure    = 3,
    ProcessorInfo      = 4,
    MemModuleInfo      = 6,
    CacheInfo          = 7,
    OemString          = 11,
    MemoryDevice       = 17,
    MemoryArrayMappedAddress = 19,
    BuiltInPointDevice = 21,
    PortableBattery    = 22,
    Break              = 127,
}

#[derive(Debug)]
#[repr(C)]
pub struct SmbiosProcessorInfo
{
    pub header:         SmbiosHeader,
    pub socket:         u8,
    pub processor_type: u8,
    pub family:         u8,
    pub manufacturer:   u8,
    pub id:             u64,
    pub version:        u8,
    pub voltage:        u8,
    pub ext_clock:      u16,
    pub max_speed:      u16,
    pub current_speed:  u16,
}

#[derive(Debug)]
#[repr(C)]
pub struct SmbiosBoardInfo
{
    pub header:              SmbiosHeader,
    pub manufacturer:        u8,
    pub product:             u8,
    pub version:             u8,
    pub serial_number:       u8,
    pub asset_tag:           u8,
    pub feature_flags:       u8,
    pub location_in_chassis: u8,
    pub chassis_handle:      u16,
    pub board_type:          u8,
    pub num_obj_handle:      *mut u16,
}

const ACPI: u32 = 1094930505;
const RSMB: u32 = 1381190978;

fn main()
{
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1);

    let mut buffer: Vec<u8> = Vec::new();

    let size = unsafe { GetSystemFirmwareTable(RSMB, 0, null_mut(), 0) };
    buffer.resize(size as _, 0);

    let _ = unsafe { GetSystemFirmwareTable(RSMB, 0, buffer.as_ptr() as _, size) };

    dbg!(size);
    dbg!(buffer.len());

    let smbios: *const Smbios = unsafe { transmute(buffer.as_ptr()) };
    let smbios = unsafe { smbios.as_ref().unwrap() };

    dbg!(&smbios);

    if mode.is_some_and(|&x| x == "dump")
    {
        let mut file = File::create("smbios_dump.bin").unwrap();
        file.write_all(&buffer).unwrap();
        return;
    }
    let mut start = addr_of!(smbios.table_data) as *mut u8;
    let end = start as usize + smbios.length as usize;
    //println!("{:x?}", addr_of!(start));

    loop
    {
        let header: *mut SmbiosHeader = unsafe { transmute(start as *mut u8) };
        //println!("header: {:x?}", header);
        let header = unsafe { &*header };
        // TODO: somehow add a sanity check so that it doesnt shit itself after systemenclosure

        match header.header_type
        {
            SmbiosHeaderType::ProcessorInfo =>
            {
                let proc_info: *mut SmbiosProcessorInfo = unsafe { transmute(header) };
                let proc_info = unsafe { &*proc_info };
                dbg!(proc_info); //TODO: Handle strings
            }
            SmbiosHeaderType::BoardInfo =>
            {
                let board_info: *mut SmbiosBoardInfo = unsafe { transmute(header) };
                let board_info = unsafe { &*board_info };
                dbg!(board_info); //TODO: Handle strings
            }
            SmbiosHeaderType::Break if header.length == 4 => break,
            _ =>
            {
                //TODO: Handle other cases? (datadump?)
            }
        }

        let mut next = (start as usize + header.length as usize) as *mut u8;
        unsafe {
            while (*next | *((next as usize + 1) as *mut u8) as u8) != 0
            {
                next = (next as usize + 1usize) as _;
                //println!("next: {:x?}", next);
            }
        }

        next = (next as usize + 2) as _;
        if next as usize >= end
        {
            break;
        }
        //println!("next: {:x?}", next);
        start = next;
    }
}
