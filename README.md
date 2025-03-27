# MiniRouter

## Bootloader

### Required Software

- [Rust](https://www.rust-lang.org/learn/get-started) (you will need the nightly toolchain)
- [QEMU/KVM](https://www.qemu.org/)

### Running the bootloader

We will be running the bootloader in QEMU. Follow the instructions [here](https://rust-osdev.github.io/uefi-rs/tutorial/vm.html) to install and launch QEMU with our custom bootloader!

#### Updated Running Instructions
- Install QEMU
- move to the bootloader directory
- Within src/main.rs, there is a comment with a command
    - this builds the project, copies the bootloader file, and runs QEMU
- Run the command in a bash terminal. It should build the project and run it.

##### Running the kernel with the bootloader
- run cargo build in the kernel directory
- run `cp target/x86_64-kernel/debug/kernel ../bootloader/esp/EFI/router_os/kernel.bin`
    - this copies the kernel file into the desired location in the filesystem that UEFI will bring up
- run the command to run the bootloader, and it should run.
