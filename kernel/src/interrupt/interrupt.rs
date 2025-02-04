pub fn enable_interrupts() {
    unsafe {
        //sets the interrupt flag
        asm!("sti");
    }
}

pub fn disable_interrupts() {
    unsafe {
        //clears the interrupt flag
        asm!("cli")
    }
}
