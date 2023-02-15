#![feature(is_some_with)]
#![feature(pointer_byte_offsets)]
use std::{
    env,
    fs::File,
    io::Write,
    mem::transmute,
    ptr::{addr_of, null_mut},
};

use packed_struct::{
    prelude::PackedStruct,
    types::{bits::Bits, ReservedZeroes},
    PackedStructSlice,
};
use winapi::um::sysinfoapi::GetSystemFirmwareTable;
// TODO: move these to another module and make them private after writing better structs for final serialization
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
    pub header:            SmbiosHeader,
    pub socket:            u8,
    pub processor_type:    u8,
    pub family:            u8,
    pub manufacturer:      u8,
    pub id:                u64,
    pub version:           u8,
    pub voltage:           u8,
    pub ext_clock:         u16,
    pub max_speed:         u16,
    pub current_speed:     u16,
    pub status:            u8,
    pub processor_upgrade: u8,
}

#[derive(Debug)]
#[repr(C)]
pub struct SmbiosBiosInfo
{
    pub header:                   SmbiosHeader,
    pub vendor:                   u8,
    pub version:                  u8,
    pub starting_address_segment: u16,
    pub release_date:             u8,
    pub rom_size:                 u8,
    pub characteristics:          BiosCharacteristics,
    pub extension:                u8,
    pub extension2:               u8,
    pub system_major_release:     u8,
    pub system_minor_release:     u8,
    pub embedded_major_release:   u8,
    pub embedded_minor_release:   u8,
    pub extended_rom_size:        u16,
}
// ! I HAVE NO IDEA WHAT IM DOING!
#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0")]
#[packed_struct(size_bytes = "8")]
pub struct BiosCharacteristics
{
    #[packed_field(bits = "0..=1")]
    pub reserved: ReservedZeroes<Bits<2>>,
    #[packed_field(bits = "2")]
    pub unknown: bool,
    #[packed_field(bits = "3")]
    pub characteristics_unsupported: bool,
    #[packed_field(bits = "4")]
    pub isa_supported: bool,
    #[packed_field(bits = "5")]
    pub mca_supported: bool,
    #[packed_field(bits = "6")]
    pub eisa_supported: bool,
    #[packed_field(bits = "7")]
    pub pci_supported: bool,
    #[packed_field(bits = "8")]
    pub pcmcia_supported: bool,
    #[packed_field(bits = "9")]
    pub pnp_supported: bool,
    #[packed_field(bits = "10")]
    pub apm_supported: bool,
    #[packed_field(bits = "11")]
    pub bios_upgradeable: bool,
    #[packed_field(bits = "12")]
    pub bios_shadowing_allowed: bool,
    #[packed_field(bits = "13")]
    pub vesa_supported: bool,
    #[packed_field(bits = "14")]
    pub escd_supported: bool,
    #[packed_field(bits = "15")]
    pub cdboot_supported: bool,
    #[packed_field(bits = "16")]
    pub selectboot_supported: bool,
    #[packed_field(bits = "17")]
    pub biosrom_socketed: bool,
    #[packed_field(bits = "18")]
    pub pcmciaboot_supported: bool,
    #[packed_field(bits = "19")]
    pub edd_supported: bool,
    #[packed_field(bits = "20")]
    pub floppynec_supported: bool,
    #[packed_field(bits = "21")]
    pub floppytoshiba_supported: bool,
    #[packed_field(bits = "22")]
    pub floppy360kb_supported: bool,
    #[packed_field(bits = "23")]
    pub floppy1_2mb_supported: bool,
    #[packed_field(bits = "24")]
    pub floppy720kb_supported: bool,
    #[packed_field(bits = "25")]
    pub floppy2_88mb_supported: bool,
    #[packed_field(bits = "26")]
    pub printscreenservice_supported: bool,
    #[packed_field(bits = "27")]
    pub keyboard8042_supported: bool,
    #[packed_field(bits = "28")]
    pub serialservice_supported: bool,
    #[packed_field(bits = "29")]
    pub printerservice_supported: bool,
    #[packed_field(bits = "30")]
    pub cgaservice_supported: bool,
    #[packed_field(bits = "31")]
    pub nec_pc98: bool,
    #[packed_field(bits = "32:47")]
    pub reserved_for_vendor: ReservedZeroes<Bits<16>>,
    #[packed_field(bits = "48:63")]
    pub reserved_for_system_vendor: ReservedZeroes<Bits<16>>,
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
    pub chassis_handle:      u8,
    packing:                 u8, // ! it seems like the chassishandle field should be an u16 but that does not work in my machine for some reason, so unless smbiosexplorer is wrong this is needed
    pub board_type:          u8,
    pub num_obj_handle:      u8,
    pub p_obj_handle:        *mut u8,
}
// i dont know how to do this conversion in rust so i did it in c++ and just copied the int values
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

    loop
    {
        let header: *mut SmbiosHeader = unsafe { transmute(start as *mut u8) };
        let str_section = unsafe { header as usize + (*header).length as usize } as *mut u8;
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
            SmbiosHeaderType::Bios =>
            {
                let bios_info: *mut SmbiosBiosInfo = unsafe { transmute(header) };
                let bios_info = unsafe { &*bios_info };
                dbg!(bios_info);
                let strct = println!(
                    "characteristics: {:#x?}",
                    bios_info.characteristics
                );
            }
            SmbiosHeaderType::Break if header.length == 4 => break, // End-of-Table
            _ =>
            {
                //TODO: Handle other cases? (datadump?)
            }
        }

        let mut next = unsafe { start.byte_add(header.length.into()) };
        unsafe {
            while (next.read() | next.byte_add(1).read()) != 0
            {
                next = next.byte_add(1);
            }
            next = next.byte_add(2);
        }

        if next as usize >= end
        {
            break;
        }

        start = next;
    }
}
