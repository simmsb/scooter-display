// const HEAP_SIZE: usize = 512;
// static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

#[global_allocator]
static ALLOCATOR: embedded_alloc::LlffHeap = embedded_alloc::LlffHeap::empty();

pub fn init() {
    unsafe {
        embedded_alloc::init!(ALLOCATOR, 512);
        // ALLOCATOR.init(core::ptr::addr_of!(HEAP) as *const u8 as usize, HEAP_SIZE);
    }
}
