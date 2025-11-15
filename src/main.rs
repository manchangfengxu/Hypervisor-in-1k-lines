#![no_std]
#![no_main]
mod allocator;
mod guest_page_table;
mod trap;
mod vcpu;
#[macro_use]
mod print;
use crate::{
    allocator::alloc_pages,
    guest_page_table::{GuestPageTable, PTEFlags},
    vcpu::VCpu,
};
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
    println!("\nBooting hypervisor...");
    // Fill the BSS section with zeros.
    init_bss();
    unsafe {
        // Set stvec with Direct mode (lower 2 bits = 00)
        let stvec_addr = (trap::trap_handler as usize) & !0b11; // Clear lower 2 bits
        asm!("csrw stvec, {}", in(reg) stvec_addr);
        // asm!("unimp"); // Illegal instruction here!
    }
    // println!("\nBooting hypervisor...");

    allocator::GLOBAL_ALLOCATOR.init(&raw mut __heap, &raw mut __heap_end);

    let kernel_image = include_bytes!("../guest.bin");
    let guest_entry = 0x100000;

    let kernel_memory = alloc_pages((kernel_image.len() + 4095) & !4095);
    unsafe {
        core::ptr::copy_nonoverlapping(kernel_image.as_ptr(), kernel_memory, kernel_image.len());
    }
    let mut table = GuestPageTable::new();
    table.map(guest_entry, kernel_memory as u64, PTEFlags::RWX);

    let mut vcpu = VCpu::new(&table, guest_entry);
    vcpu.run();
}

use core::panic::PanicInfo;

#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    println!("PANIC: {}", info);
    loop {
        unsafe {
            core::arch::asm!("wfi"); // Wait for an interrupt (idle loop)
        }
    }
}
