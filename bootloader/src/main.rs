#![no_std]
#![no_main]
mod cfg_table_type;
mod identify_acpi_handler;
mod kernel_args;
use core::slice;

use acpi::{AcpiTables, PciConfigRegions};
use cfg_table_type::CfgTableType;
use identify_acpi_handler::IdentityAcpiHandler;
use kernel_args::KernelArgs;

use log::info;
use uefi::boot::{self, SearchType};
use uefi::prelude::*;
//use uefi::proto::console::text::Output;
use uefi::proto::device_path::text::{AllowShortcuts, DevicePathToText, DisplayOnly};
use uefi::proto::loaded_image::LoadedImage;
use uefi::system::with_config_table;
use uefi::table::cfg::ConfigTableEntry;
use uefi::table::system_table_raw;
use uefi::{Identify, Result};
use uefi_raw::protocol::console::SimpleTextInputProtocol;
use uefi_raw::table::system::SystemTable;

#[entry]
fn uefi_main() -> Status {
    uefi::helpers::init().unwrap();
    print_image_path().unwrap();
    let st = unsafe {
        system_table_raw()
            .expect("Unable to obtain system table pointer.")
            .as_mut()
    };

    info!("Hello world!");
    boot::stall(20_000_000);

    let mut karg = KernelArgs::default();
    st.configuration_table;
    info!("Empty karg: {:?}", karg);
    //karg.populate_from_cfg_table(enumerate_config_table(st));
    info!("Populated karg: {:?}", karg);

    let ih = IdentityAcpiHandler; // Create a new IdentityAcpiHandler
    let acpi_tables = unsafe { AcpiTables::from_rsdp(ih, karg.get_acpi().0 as usize) }.unwrap();

    let pcie_cfg = PciConfigRegions::new(&acpi_tables).unwrap();
    for sg in 0u16..=65535u16 {
        if let Some(addr) = pcie_cfg.physical_address(sg, 0, 0, 0) {
            karg.set_pcie(addr as *mut core::ffi::c_void);
            break;
        }
    }
    info!("karg after PCIe: {:?}", karg);

    //let (mm_ptr, count) = get_mm(&system_table);
    //karg.set_memmap(mm_ptr, count);

    info!("Got memory");
    info!("karg after MemMap: {:?}", karg);

    //info!("Press any key to continue...");

    //let stdin = unsafe {
    //    let stdin: *mut SimpleTextInputProtocol = st.stdin;
    //    ((*stdin).reset)(stdin, true);
    //    stdin
    //};
    //
    //let mut key_press_event = unsafe { [st.stdin.wait_for_key_event().unsafe_clone()] };
    //st.boot_services()
    //    .wait_for_event(&mut key_press_event)
    //    .expect("Did not receive keypress.");

    Status::SUCCESS
}

fn print_image_path() -> Result {
    let loaded_image = boot::open_protocol_exclusive::<LoadedImage>(boot::image_handle())?;

    let device_path_to_text_handle =
        *boot::locate_handle_buffer(SearchType::ByProtocol(&DevicePathToText::GUID))?
            .first()
            .expect("DevicePathToText is missing");

    let device_path_to_text =
        boot::open_protocol_exclusive::<DevicePathToText>(device_path_to_text_handle)?;
    let image_device_path = loaded_image.file_path().expect("File path is not set!");
    let image_device_path_text = device_path_to_text
        .convert_device_path_to_text(image_device_path, DisplayOnly(true), AllowShortcuts(false))
        .expect("convert_device_path_to_text failed");

    info!("Image path: {}", &*image_device_path_text);
    Ok(())
}

//fn enumerate_config_table(st: &SystemTable) -> &[ConfigTableEntry] {
//    let num_entries = st.number_of_configuration_table_entries;
//
//    unsafe {
//        let slice = slice::from_raw_parts(st.configuration_table, num_entries);
//        slice.into()
//    }
//}
