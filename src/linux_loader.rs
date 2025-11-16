use crate::allocator::alloc_pages;
use crate::guest_page_table::{GuestPageTable, PTEFlags};
use crate::println;
use core::mem::size_of;
use alloc::format;
use alloc::vec::Vec;

#[repr(C)]
struct RiscvImageHeader {
    code0: u32,
    code1: u32,
    text_offset: u64,
    image_size: u64,
    flags: PTEFlags,
    version: u32,
    reserved1: u32,
    reserved2: u64,
    magic: u64,
    magic2: u32,
    reserved3: u32,
}

pub const GUEST_BASE_ADDR: u64 = 0x80200000;
pub const MEMORY_SIZE: usize = 64 * 1024 * 1024;
pub const GUEST_DTB_ADDR: u64 = 0x7000_0000;

fn copy_and_map(
    table: &mut GuestPageTable,
    data: &[u8],
    guest_addr: u64,
    len: usize,
    flags: PTEFlags,
) {
    // Allocate a memory region, and copy the data to it.
    assert!(data.len() <= len, "data is beyond the region");
    let raw_ptr = alloc_pages(len);
    unsafe {
        core::ptr::copy_nonoverlapping(data.as_ptr(), raw_ptr, data.len());
    }
    // Map the memory region to the guest's address space.
    let host_ptr = raw_ptr as u64;
    for off in (0..len).step_by(4096) {
        table.map(guest_addr + off as u64, host_ptr + off as u64, flags);
    }
}

pub fn load_linux_kernel(table: &mut GuestPageTable, image: &[u8]) {
    assert!(image.len() >= size_of::<RiscvImageHeader>());
    let header = unsafe { &*(image.as_ptr() as *const RiscvImageHeader) };
    assert_eq!(u32::from_le(header.magic2), 0x05435352, "invalid magic");
    println!("text_offset = {:#x}", u64::from_le(header.text_offset));
    // println!(
    //     "entry = {:#x}",
    //     GUEST_BASE_ADDR + u64::from_le(header.text_offset)
    // );
    let kernel_size = u64::from_le(header.image_size);
    assert!(image.len() <= MEMORY_SIZE);

    copy_and_map(table, image, GUEST_BASE_ADDR, MEMORY_SIZE, PTEFlags::RWX);

    let dtb = build_device_tree().unwrap();
    assert!(dtb.len() <= 0x10000, "DTB is too large");
    copy_and_map(table, &dtb, GUEST_DTB_ADDR, dtb.len(), PTEFlags::R);
    println!("loaded kernel: size={}KB", kernel_size / 1024);
}

fn build_device_tree() -> Result<Vec<u8>, vm_fdt::Error> {
    let mut fdt = vm_fdt::FdtWriter::new()?;
    let root_node = fdt.begin_node("")?;
    fdt.property_string("compatible", "riscv-virtio")?;
    fdt.property_u32("#address-cells", 0x2)?;
    fdt.property_u32("#size-cells", 0x2)?;

    let chosen_node = fdt.begin_node("chosen")?;
    fdt.property_string("bootargs", "console=hvc earlycon=sbi panic=-1")?;
    fdt.end_node(chosen_node)?;

    let memory_node = fdt.begin_node(&format!("memory@{}", GUEST_BASE_ADDR))?;
    fdt.property_string("device_type", "memory")?;
    fdt.property_array_u64("reg", &[GUEST_BASE_ADDR, MEMORY_SIZE as u64])?;
    fdt.end_node(memory_node)?;

    let cpus_node = fdt.begin_node("cpus")?;
    fdt.property_u32("#address-cells", 0x1)?;
    fdt.property_u32("#size-cells", 0x0)?;
    fdt.property_u32("timebase-frequency", 10000000)?;

    let cpu_node = fdt.begin_node("cpu@0")?;
    fdt.property_string("device_type", "cpu")?;
    fdt.property_string("compatible", "riscv")?;
    fdt.property_u32("reg", 0)?;
    fdt.property_string("status", "okay")?;
    fdt.property_string("mmu-type", "riscv,sv48")?;
    fdt.property_string("riscv,isa", "rv64imafdc")?;

    fdt.end_node(cpu_node)?;
    fdt.end_node(cpus_node)?;
    fdt.end_node(root_node)?;
    fdt.finish()
}
