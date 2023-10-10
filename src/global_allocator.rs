use alloc_cortex_m::CortexMHeap;

#[global_allocator]
pub static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

// 在初始化函数或`main`函数的开始位置，指定堆的大小和位置
pub fn init_allocator() {
    // Please set the correct heap size.
    const HEAP_SIZE: usize = 0x20000;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    unsafe { ALLOCATOR.init(HEAP.as_ptr() as usize, HEAP.len()) }
}
