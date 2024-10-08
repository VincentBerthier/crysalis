use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};

/// Initialize a new `OffsetPageTable`.
///
/// # Safety
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
#[must_use]
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// Returns a mutable reference to the active level 4 table.
///
/// # Safety
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
#[must_use]
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe {
        &mut *page_table_ptr // unsafe
    }
}

// /// Creates an example mapping for the given page to frame `0xb8000`.
// pub fn create_example_mapping<F>(page: Page, mapper: &mut OffsetPageTable, frame_allocator: &mut F)
// where
//     F: FrameAllocator<Size4KiB>,
// {
//     let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
//     let flags = Flags::PRESENT | Flags::WRITABLE;

//     let map_to_result = unsafe {
//         // FIXME: this is not safe, we do it only for testing
//         mapper.map_to(page, frame, flags, frame_allocator)
//     };
//     map_to_result.expect("map_to failed").flush();
// }

/// A `FrameAllocator` that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        self.memory_map
            .iter()
            // get usable regions from memory map
            .filter(|region| region.region_type == MemoryRegionType::Usable)
            // map each region to its address range
            .map(|region| region.range.start_addr()..region.range.end_addr())
            // transform to an iterator of frame start addresses
            .flat_map(|region| region.step_by(4096))
            // create `PhysFrame` types from the start addresses
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

impl BootInfoFrameAllocator {
    /// Create a `FrameAllocator` from the passed memory map.
    ///
    /// # Safety
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    #[must_use]
    pub const unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        Self {
            memory_map,
            next: 0,
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
