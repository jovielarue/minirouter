#![no_std]
#![no_main]

mod modules { pub mod uefi; }
use modules::uefi::*;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "efiapi" fn efi_main(_handle: ImageHandle, _system_table:*const SystemTable) {
    let string: &str = "hello\n\r";
    for character in string.chars(){
        let mut buffer:[u16;1] = [0];
        let utf16: &mut [u16] = character.encode_utf16(&mut buffer);
        
        unsafe{
            let _status = ((*(*_system_table).output)
                .output_string)((*_system_table).output, &utf16[0],);
        }
    }

    //using the char encoding method
    let string = "hello\n\r";
    for character in string.chars(){
    
        let mut buffer:[u16;1] = [0];
        let utf16 = character.encode_utf16(&mut buffer);
        
        unsafe{
            let _status = ((*(*_system_table).output)
                .output_string)((*_system_table).output, &utf16[0],);
        }
    }
    
    //using the char casting method

    let string_arr = ['h' as u16, 'i' as u16, '!' as u16, '\n' as u16, '\0' as u16];
    
    unsafe{
        let _status = ((*(*_system_table).output)
            .output_string)((*_system_table).output, &string_arr[0],);
    }


    // This is the entry point of our kernel
    loop{}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
