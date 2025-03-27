#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;

const HELLO: &[u8] = b"Hello World!";

#[unsafe(no_mangle)] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {

    //get the address of the vga buffer
    let vga_buffer = 0xb8000 as *mut u8;

    //loop through the HELLO ascii byte array, and print write it to the vga buffer
    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte; //ascii byte
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb; //color
        }
    }

    //loop continuously.
    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
