//found from
//https://medium.com/@applepies12/writing-an-os-in-rust-part-2-8a424f71e6ae

pub type ImageHandle = u64;
pub type Status = usize;

#[repr(C)]
pub struct SystemTable{
    header: [u8;24],
    firmware_vendor: u64,
    firmware_revision:u32,
    input_handle: ImageHandle,
    input: u64,
    output_handle: ImageHandle,
    pub output: *const TextOutputProtocol,
    error_handle: ImageHandle,
    error:  u64,
    runtime: u64,
    boot: u64,
    no_of_entries: usize,
    config_table: u64,
}

//u64 are pointers to functions that will not be used
#[repr(C)]
pub struct TextOutputProtocol{
    reset: u64,
    pub output_string: OutputString,
    test_output: u64,
    query_mode: u64,
    set_mode: u64,
    set_attribute: u64,
    clear_screen:  u64,
    set_cursor_position: u64,
    enable_cursor: u64,
    mode: u64,
}

type OutputString = extern "efiapi" fn(
    output_protocol:*const TextOutputProtocol,
    string :*const u16
)-> Status;
