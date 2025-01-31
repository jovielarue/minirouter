//File implemented with help from the following chat:
//https://chatgpt.com/share/679bedc9-36c0-800c-869e-e537c23eb7c9

/*
 * A struct to store IDT Entries within. this will eventually help to build up the interrrupt
 * descriptor table, which is fundamental for working with interrupts
 *
 * Built with help from:
 * https://wiki.osdev.org/Interrupt_Descriptor_Table
 */
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct IDTEntry {
    pub offset_1: u16,  // The first offset (bits 0-15)
    pub selector: u16,  // Segment selector. Points to a valid spot in the GDT
    pub ist: u8,        // An offset to the interrupt stack table (not used if all bits 0)
    pub type_attrs: u8, // The gate type, and various other attributes (dpl & p fields)
    pub offset_2: u16,  // The second offset (bits 16-31)
    pub offset_3: u32,  // The third offset (bits 32-63)
    pub zero: u32       // A whole bunch of zeroes
}

/*
 * The pointer for the cpu to determine the location and size of the IDT in memory.
 */
#[repr(C)]
pub struct IDTPointer {
    pub limit: u16,
    pub base: u64
}
