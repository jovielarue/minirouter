#![no_std]
#![no_main]

/**
 *  cargo build \
    && cp target/x86_64-unknown-uefi/debug/bootloader.efi esp/EFI/BOOT/bootx64.efi \
    && qemu-system-x86_64 -enable-kvm \
     -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.fd \
     -drive if=pflash,format=raw,readonly=on,file=OVMF_VARS.fd \
     -drive format=raw,file=fat:rw:esp
 */

extern crate alloc;

use uefi::allocator::Allocator;

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

use log::info;
use alloc::vec::Vec;
use uefi::boot::MemoryType;
use uefi::prelude::*;
use uefi::CString16;
use uefi::fs::FileSystem;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::boot::{self, ScopedProtocol};
use uefi::Error;

const KERNEL_LOCATION: &str = "\\EFI\\router_os\\kernel.bin"; 

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    info!("Hello world!");
    
    //attempt to convert the kernel location to a cstring.
    let path: CString16 = CString16::try_from(KERNEL_LOCATION).unwrap();

    //read in the kernel file and store it in a buffer
    let buffer: Vec<u8> = match read_in_kernel(path) {
        Ok(buff) => buff,
        Err(e) => {
            info!("ERROR: could not load kernel: {:?}", e);
            return Status::LOAD_ERROR;
        }
    };

    info!("Kernel file loaded: {} bytes", buffer.len());

    //allocate memory for the kernel, and get the address
    
    if let Some(kernel_addr) = allocate_kernel_mem(&buffer) {
        info!("Kernel address: {:p}", kernel_addr);
    
        unsafe {
            let entry_fn: extern "C" fn() -> ! = core::mem::transmute(kernel_addr);
            
            // let addr = kernel_addr;
            // info!("Bytes after entry point:");
            // for i in 0..16 {
            //     info!("{:#x}: {:#x}", addr.offset(i) as usize, *addr.offset(i));
            // }
                
            info!("Entering entry function now...");
            // let _mem = boot::exit_boot_services(MemoryType::LOADER_DATA);

            entry_fn();
        }
    } else {
        info!("The kernel failed to be parsed");
        Status::ABORTED
    }

}

fn read_in_kernel(path: CString16) -> Result<Vec<u8>, Error> {
    //open the filesystem to the root
    let fs_handle: ScopedProtocol<SimpleFileSystem> = boot::get_image_file_system(boot::image_handle()).unwrap();
    let mut fs: FileSystem = FileSystem::new(fs_handle);

    //attempt to open the kernel binary
    let buffer: Vec<u8>  = fs.read(path.as_ref()).unwrap();

    Ok(buffer)
}

fn allocate_kernel_mem(buffer: &[u8]) -> Option<*const u8> {
    
    if let Some(header) = parse_elf_header(buffer) {
        //load the segments into memory
        match load_segments(
            buffer,
            header
        ) {
            Ok(_) => Some(header.e_entry as *const u8), //return the entry function address
            Err(e) => {
                info!("Error allocating kernel memory: {}", e);
                return None;
            }
        }
        
    } else {
        info!("failed to parse elf header");
        return None;
    }
}


#[repr(C)]
struct Elf64Ehdr {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

// Function to parse the ELF header
fn parse_elf_header(data: &[u8]) -> Option<&Elf64Ehdr> {
    // Check ELF magic number
    if &data[0..4] != b"\x7fELF" {
        return None;
    }

    let header: &Elf64Ehdr = unsafe { &*(data.as_ptr() as *const Elf64Ehdr) };
    //wrap the header in an option
    Some(header)
}





#[repr(C)]
struct Elf64Phdr {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

// Function to parse program headers and load segments
fn load_segments(elf_data: &[u8], e_header: &Elf64Ehdr) -> Result<(), uefi::Error> {

    // Loop through all program headers
    info!("number of program headers: {}", e_header.e_phnum);

    //parse the program headers
    //if None was returned, abort
    if let Some(p_headers) = parse_program_headers(elf_data, e_header) {

        match load_elf_segments(elf_data, p_headers) {
            Ok(_) => (),
            Err(msg) => return Err(msg)
        }
    } else {
        return Err(uefi::Error::new(Status::LOAD_ERROR, ()));
    }

    Ok(())
}

fn parse_program_headers<'a>(buffer: &'a [u8], e_header: &'a Elf64Ehdr) -> Option<&'a [Elf64Phdr]> {
    
    let phoff = e_header.e_phoff as usize;
    let phsize = e_header.e_phentsize as usize;
    let phnum = e_header.e_phnum as usize;
    
    if buffer.len() < ((phoff + phnum * phsize)) {
        //parsing the headers failed. the buffer isn't large enough.
        return None;
    }

    //return the list of parsed program headers
    Some(unsafe {
        core::slice::from_raw_parts(
            buffer.as_ptr().add(e_header.e_phoff as usize) as *const Elf64Phdr,
            e_header.e_phnum as usize,
        )
    })
}


const PAGE_SIZE: usize = 4096;
const LOAD_SEGMENT_TYPE: u32 = 1;

fn load_elf_segments(buffer: &[u8], ph_table: &[Elf64Phdr]) -> Result<(), uefi::Error> {
    //loop through the program headers found and display their information
    //this will be where we would actually load the segments into memory
    for (i, ph) in ph_table.iter().enumerate() {
        info!("PH {}: Type = {}, Offset = 0x{:x}, VAddr = 0x{:x}, memsz: {}, endAddr: 0x{:x}",
            i, ph.p_type, ph.p_offset, ph.p_vaddr, ph.p_memsz,
            unsafe { (ph.p_vaddr as *const u8).offset(ph.p_memsz.try_into().unwrap()) as u64 }
        );
        
        //only the segments labelled LOAD need to be loaded
        if ph.p_type != LOAD_SEGMENT_TYPE {
            continue;
        }

        info!("loading segment {} into memory...", i);

        //now, rather than just displaying data to the screen, we need to load these segemnts
        //into memory
        let vaddr = ph.p_vaddr as usize;
        let offset = ph.p_offset as usize;
        let filesz = ph.p_filesz as usize;
        let memsz = ph.p_memsz as usize;

        //align the start and end with the page size
        let page_aligned_start = vaddr & !(PAGE_SIZE - 1);
        let page_aligned_end = (vaddr + memsz + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);

        //this will be a whole number, as the start and end have been aligned to the page size
        let num_pages = (page_aligned_end - page_aligned_start) / PAGE_SIZE;

        let allocated_addr = match boot::allocate_pages(
            boot::AllocateType::Address(page_aligned_start as u64),
            MemoryType::LOADER_DATA,
            num_pages
        ) {
            Ok(addr) => addr,
            Err(msg) => {
                info!("ERROR: {}", msg);
                return Err(msg);
            }
        };

        let dest_ptr = allocated_addr.as_ptr();

        //Calculate the amount of padding needed for the given segment
        let padding_start = page_aligned_start + (vaddr % PAGE_SIZE);

        if padding_start > page_aligned_start {
            unsafe {
                core::ptr::write_bytes(
                    dest_ptr,
                    0,
                    padding_start - page_aligned_start
                );
            }
        }

        //copy the segment into the allocated memory
        unsafe {
            //copy the segment from the buffer to the destination
            core::ptr::copy_nonoverlapping(
                buffer.as_ptr().add(offset),
                dest_ptr.add(padding_start - page_aligned_start),
                filesz
            );

            //Zero out the .bss section (extra memory)
            let bss_start = dest_ptr.add(padding_start - page_aligned_start + filesz);
            core::ptr::write_bytes(bss_start, 0, memsz - filesz);
        }

        info!("loaded segment at 0x{:x} with {} pages, size: {} bytes (mem size: {} bytes)",
            vaddr, num_pages, filesz, memsz
        );
    }

    //return 
    Ok(())
}


// mod cfg_table_type;
// mod identify_acpi_handler;
// mod kernel_args;
// use core::slice;

// use acpi::{AcpiTables, PciConfigRegions};
// use cfg_table_type::CfgTableType;
// use identify_acpi_handler::IdentityAcpiHandler;
// use kernel_args::KernelArgs;

// use log::info;
// use uefi::boot::{self, SearchType};
// use uefi::prelude::*;
// //use uefi::proto::console::text::Output;
// use uefi::proto::device_path::text::{AllowShortcuts, DevicePathToText, DisplayOnly};
// use uefi::proto::loaded_image::LoadedImage;
// use uefi::system::with_config_table;
// use uefi::table::cfg::ConfigTableEntry;
// use uefi::table::system_table_raw;
// use uefi::{Identify, Result};
// use uefi_raw::protocol::console::SimpleTextInputProtocol;
// use uefi_raw::table::system::SystemTable;

// #[entry]
// fn uefi_main() -> Status {
//     uefi::helpers::init().unwrap();
//     print_image_path().unwrap();
//     let st = unsafe {
//         system_table_raw()
//             .expect("Unable to obtain system table pointer.")
//             .as_mut()
//     };

//     info!("Hello world!");
//     boot::stall(20_000_000);

//     let mut karg = KernelArgs::default();
//     st.configuration_table;
//     info!("Empty karg: {:?}", karg);
//     //karg.populate_from_cfg_table(enumerate_config_table(st));
//     info!("Populated karg: {:?}", karg);

//     let ih = IdentityAcpiHandler; // Create a new IdentityAcpiHandler
//     let acpi_tables = unsafe { AcpiTables::from_rsdp(ih, karg.get_acpi().0 as usize) }.unwrap();

//     let pcie_cfg = PciConfigRegions::new(&acpi_tables).unwrap();
//     for sg in 0u16..=65535u16 {
//         if let Some(addr) = pcie_cfg.physical_address(sg, 0, 0, 0) {
//             karg.set_pcie(addr as *mut core::ffi::c_void);
//             break;
//         }
//     }
//     info!("karg after PCIe: {:?}", karg);

//     //let (mm_ptr, count) = get_mm(&system_table);
//     //karg.set_memmap(mm_ptr, count);

//     info!("Got memory");
//     info!("karg after MemMap: {:?}", karg);

//     //info!("Press any key to continue...");

//     //let stdin = unsafe {
//     //    let stdin: *mut SimpleTextInputProtocol = st.stdin;
//     //    ((*stdin).reset)(stdin, true);
//     //    stdin
//     //};
//     //
//     //let mut key_press_event = unsafe { [st.stdin.wait_for_key_event().unsafe_clone()] };
//     //st.boot_services()
//     //    .wait_for_event(&mut key_press_event)
//     //    .expect("Did not receive keypress.");

//     Status::SUCCESS
// }

// fn print_image_path() -> Result {
//     let loaded_image = boot::open_protocol_exclusive::<LoadedImage>(boot::image_handle())?;

//     let device_path_to_text_handle =
//         *boot::locate_handle_buffer(SearchType::ByProtocol(&DevicePathToText::GUID))?
//             .first()
//             .expect("DevicePathToText is missing");

//     let device_path_to_text =
//         boot::open_protocol_exclusive::<DevicePathToText>(device_path_to_text_handle)?;
//     let image_device_path = loaded_image.file_path().expect("File path is not set!");
//     let image_device_path_text = device_path_to_text
//         .convert_device_path_to_text(image_device_path, DisplayOnly(true), AllowShortcuts(false))
//         .expect("convert_device_path_to_text failed");

//     info!("Image path: {}", &*image_device_path_text);
//     Ok(())
// }

//fn enumerate_config_table(st: &SystemTable) -> &[ConfigTableEntry] {
//    let num_entries = st.number_of_configuration_table_entries;
//
//    unsafe {
//        let slice = slice::from_raw_parts(st.configuration_table, num_entries);
//        slice.into()
//    }
//}
