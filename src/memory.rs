use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};
use x86_64::VirtAddr;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub const HEAP_START: u64 = 0x_4444_4444_0000;
/// Size allocated for the kernel’s heap
pub const HEAP_SIZE: u64 = 100 * 1024; // 100 KiB

/// Initializes the global allocator
///
/// # Errors
/// If the allocator couldn’t be initialized
pub fn init<F, M>(mapper: &mut M, frame_allocator: &mut F) -> Result<(), MapToError<Size4KiB>>
where
    M: Mapper<Size4KiB>,
    F: FrameAllocator<Size4KiB>,
{
    let heap_start = VirtAddr::new(HEAP_START);
    let heap_end = heap_start + HEAP_SIZE - 1_u64;
    let heap_start_page = Page::containing_address(heap_start);
    let heap_end_page = Page::containing_address(heap_end);
    let page_range = Page::range_inclusive(heap_start_page, heap_end_page);

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    unsafe {
        #[expect(clippy::cast_possible_truncation, reason = "heap is 100KiB")]
        ALLOCATOR
            .lock()
            .init(heap_start.as_mut_ptr(), HEAP_SIZE as usize);
    }

    Ok(())
}
