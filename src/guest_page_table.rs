use bitflags::bitflags;
use core::mem::size_of;

bitflags! {
    /// Page Table Entry flags for RISC-V
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PTEFlags: u64 {
        const V = 1 << 0;  // Valid
        const R = 1 << 1;  // Readable
        const W = 1 << 2;  // Writable
        const X = 1 << 3;  // Executable
        const U = 1 << 4;  // User accessible
        const G = 1 << 5;  // Global
        const A = 1 << 6;  // Accessed
        const D = 1 << 7;  // Dirty

        // Common combinations
        const RW = Self::R.bits() | Self::W.bits();
        const RX = Self::R.bits() | Self::X.bits();
        const RWX = Self::R.bits() | Self::W.bits() | Self::X.bits();
    }
}

const PPN_SHIFT: usize = 12;
const PTE_PPN_SHIFT: usize = 10;

/// Page Table Entry
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct Entry(u64);

impl Entry {
    /// Create a new PTE from physical address and flags
    pub fn new(paddr: u64, flags: PTEFlags) -> Self {
        let ppn = paddr >> PPN_SHIFT;
        Self((ppn << PTE_PPN_SHIFT) | flags.bits())
    }

    /// Check if the entry is valid
    pub fn is_valid(&self) -> bool {
        self.flags().contains(PTEFlags::V)
    }

    /// Get the physical address from this entry
    pub fn paddr(&self) -> u64 {
        (self.0 >> PTE_PPN_SHIFT) << PPN_SHIFT
    }

    /// Get the flags from this entry
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits_truncate(self.0 & 0xFF)
    }
}

/// Page Table (512 entries)
#[repr(transparent)]
struct Table([Entry; 512]);

impl Table {
    /// Allocate a new page table
    pub fn alloc() -> *mut Table {
        crate::allocator::alloc_pages(size_of::<Table>()) as *mut Table
    }

    /// Get the entry for a given guest physical address at a specific level
    pub fn entry_by_addr(&mut self, guest_paddr: u64, level: usize) -> &mut Entry {
        let index = (guest_paddr >> (12 + 9 * level)) & 0x1ff; // extract 9-bits index
        &mut self.0[index as usize]
    }
}

/// Guest Page Table (for Stage-2 translation)
pub struct GuestPageTable {
    table: *mut Table,
}

impl GuestPageTable {
    /// Create a new guest page table
    pub fn new() -> Self {
        Self {
            table: Table::alloc(),
        }
    }

    /// Get the hgatp CSR value for this page table
    /// Format: MODE[63:60] | VMID[57:44] | PPN[43:0]
    pub fn hgatp(&self) -> u64 {
        (9u64 << 60/* Sv48x4 */) | ((self.table as u64) >> PPN_SHIFT)
    }

    /// Map a guest physical address to a host physical address
    pub fn map(&mut self, guest_paddr: u64, host_paddr: u64, flags: PTEFlags) {
        let mut table = unsafe { &mut *self.table };

        // Walk through levels 3, 2, 1 to create intermediate page tables if needed
        for level in (1..=3).rev() {
            // level = 3, 2, 1
            let entry = table.entry_by_addr(guest_paddr, level);
            if !entry.is_valid() {
                let new_table_ptr = Table::alloc();
                *entry = Entry::new(new_table_ptr as u64, PTEFlags::V);
            }

            table = unsafe { &mut *(entry.paddr() as *mut Table) };
        }

        // Map the final entry at level 0
        let entry = table.entry_by_addr(guest_paddr, 0);
        crate::println!("map: {:#010x} -> {:#010x}", guest_paddr, host_paddr);
        assert!(!entry.is_valid(), "already mapped");
        *entry = Entry::new(host_paddr, flags | PTEFlags::V | PTEFlags::U);
    }
}
