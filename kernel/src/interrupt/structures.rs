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

impl IDTEntry {
    
    fn set_handler(&mut self, handler: *const ()) {
        
        //get the address of the handler as a 64-bit unsigned int
        let handler = handler as u64;

        //split the address amongst the three offsets
        self.offset_1 = (handler & 0xFFFF) as u16; //pulling out the first 16 bits
        self.offset_2 = ((handler >> 16) & 0xFFFF) as u16; //pull out the first 16, return next 16
        self.offset_3 = (handler >> 32) as u32; //pull out the first 32, return the rest.

        //setting the other entries
        self.selector = 0x08;   //kernel code segment
        self.ist = 0;           //reserved (must be zero)
        self.type_attrs = 0x8E; //Interrupt gate, present, DPL = 0
        self.zero = 0;          //reserved (must be zero)
    }
}


/*
 * The IDT. Stores the address
 */
#[repr(C)]
pub struct Idt {
    pub limit: u16,
    pub base: u64,
    pub entries: [IDTEntry; 256],
}

impl Idt {

    pub fn new() -> Self {
        let mut idt = Idt {
            limit: (128 * 256 - 1) as u16, //The max size of the IDT
            base: 0, //The start address of the IDT
            entries: [IDTEntry { //initialize a list of 256 IDT entries
                offset_1: 0,
                selector: 0,
                ist: 0,
                type_attrs: 0,
                offset_2: 0,
                offset_3: 0,
                zero: 0,
            }; 256]
        };

        //set up a default handler for all IDT entries. they will be filled in later.
        for i in 0..256 {
            idt.entries[i].set_handler(default_handler as *const ());
        }

        idt.base = &idt.entries[0] as *const _ as u64 //store the base address of the IDT

        idt //return the idt
    }

    pub unsafe fn load(&self) {
        asm!(
            "lidt [{}]",
            in(reg) self,
            options(nostack, preserves_flags)
        )
    }
}


//handlers go here
extern "x86-interrupt" fn default_handler(stack_frame: &mut InterruptStackFrame) {
    //stuff to handle the interrupt here
    println!("handled interrupt: {:?}", stack_frame);
}
