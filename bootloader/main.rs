#![no_std]
#![no_main]

use log::info;
use uefi::boot::{self, SearchType};
use uefi::prelude::*;
//use uefi::proto::console::text::Output;
use uefi::proto::device_path::text::{AllowShortcuts, DevicePathToText, DisplayOnly};
use uefi::proto::loaded_image::LoadedImage;
use uefi::{Identify, Result};

#[entry]
fn uefi_main() -> Status {
    uefi::helpers::init().unwrap();
    print_image_path().unwrap();

    info!("Hello world 2!");
    boot::stall(10_000_000);

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

//fn print_hello_world() -> Result {
//    let text_handle = *boot::locate_handle_buffer(SearchType::ByProtocol(&Output::GUID))?
//        .first()
//        .unwrap();
//
//    let text_handle = Output::clear(text_handle);
//
//    let open_text_handle = boot::open_protocol_exclusive::<Output>(text_handle)?;
//
//    info!("Text handle: {:?}", text_handle);
//
//    Ok(())
//}
