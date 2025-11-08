#![no_std]
#![no_main]
mod allocator;
mod trap;
#[macro_use]
mod print;
use core::arch::asm;
extern crate alloc;
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot")]
pub extern "C" fn boot() -> ! {
    unsafe {
        asm!(
            "la sp, __stack_top",  // Load __stack_top address into sp
            "j {main}",            // Jump to main
            main = sym main,       // Defines {main} in the assembly code
            options(noreturn)      // No return from this function
        );
    }
}

unsafe extern "C" {
    static mut __bss: u8;
    static mut __bss_end: u8;
    static mut __heap: u8;
    static mut __heap_end: u8;
}

fn init_bss() {
    unsafe {
        let bss_start = &raw mut __bss;
        let bss_size = (&raw mut __bss_end as usize) - (&raw mut __bss as usize);
        core::ptr::write_bytes(bss_start, 0, bss_size);
    }
}

fn main() -> ! {
    // Fill the BSS section with zeros.
    init_bss();
    println!("\nBooting hypervisor...");
    allocator::GLOBAL_ALLOCATOR.init(&raw mut __heap, &raw mut __heap_end);
    println!("Allocator initialized");
    let mut hstatus: u64 = 0;
    hstatus |= 2 << 32; // VSXL: XLEN for VS-mode (64-bit)
    hstatus |= 1 << 7; // SPV: Supervisor Previous Virtualization mode

    let sepc: u64 = 0x1234abcd;
    println!("hstatus: {}", hstatus);
    println!("sepc: {}", sepc);
    unsafe {
        asm!(
            "csrw hstatus, {hstatus}",
            "csrw sepc, {sepc}",
            "sret",
            hstatus = in(reg) hstatus,
            sepc = in(reg) sepc,
        );
    }
    println!("check");
    unreachable!();
}

use core::panic::PanicInfo;

#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    println!("PANIC HANDLER ENTERED!");
    println!("PANIC: {}", info);
    loop {
        unsafe {
            core::arch::asm!("wfi"); // Wait for an interrupt (idle loop)
        }
    }
}
