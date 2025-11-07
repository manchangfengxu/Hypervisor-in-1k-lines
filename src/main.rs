#![no_std]
#![no_main]

#[macro_use]
mod print;
use core::arch::asm;

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
    println!("Hello, world!");
    // Infinite loop.
    loop {}
}

use core::panic::PanicInfo;

#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    loop {
        unsafe {
            core::arch::asm!("wfi"); // Wait for an interrupt (idle loop)
        }
    }
}