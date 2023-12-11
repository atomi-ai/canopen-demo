use alloc::string::{String, ToString};
use defmt::info;
use crate::flash::flash;
use crate::global_allocator;

const BLOCK_SIZE: usize = 4096;
const FLASH_BASE_ADDR: u32 = 0x10000000;
pub const EDS_DATA_ADDRESS: u32 = 0x10300000;

/// Reads a string from the flash.
/// The format in flash is expected to be: [4-byte length][string data]
pub fn read_string_from_flash(base_addr: u32) -> Option<String> {
    let flash_ptr = base_addr as *const u8;

    // Extract length (first 4 bytes)
    let length_bytes = unsafe { core::slice::from_raw_parts(flash_ptr, 4) };
    let string_length = u32::from_le_bytes([length_bytes[0], length_bytes[1], length_bytes[2], length_bytes[3]]) as usize;

    // Extract string bytes
    let string_bytes = unsafe { core::slice::from_raw_parts(flash_ptr.add(4), string_length) };

    // Convert bytes to string
    core::str::from_utf8(string_bytes).map(|s| s.to_string()).ok()
}

/// Writes a string to the flash.
/// The format in flash will be: [4-byte length][string data]
pub fn write_bytes_to_flash(base_addr: u32, bytes: &[u8]) -> Result<(), &'static str> {
    let total_bytes = 4 + bytes.len(); // 4 bytes for length, rest for data

    if total_bytes > BLOCK_SIZE * BLOCK_SIZE {  // Assuming the total flash size is BLOCK_SIZE*BLOCK_SIZE
        return Err("Data is too large for the flash.");
    }

    // Calculate how many blocks we'll need to use
    let blocks_needed = (total_bytes + BLOCK_SIZE - 1) / BLOCK_SIZE;  // Ceiling division

    let length_as_bytes: [u8; 4] = (bytes.len() as u32).to_le_bytes();

    for i in 0..blocks_needed {
        let block_start = i * BLOCK_SIZE;
        let block_end = core::cmp::min((i + 1) * BLOCK_SIZE, total_bytes);

        let mut buffer: [u8; BLOCK_SIZE] = [0xFF; BLOCK_SIZE];

        // If we're on the first block, insert the length
        if i == 0 {
            buffer[..4].copy_from_slice(&length_as_bytes);
            let offset = block_end - 4;  // Adjust for the length bytes
            buffer[4..block_end].copy_from_slice(&bytes[..offset]);
        } else {
            let start = block_start - 4;  // Adjust for the length bytes in the first block
            buffer[..block_end - block_start].copy_from_slice(&bytes[start..block_end - 4]);
        }

        let addr_to_write = base_addr + (i as u32 * BLOCK_SIZE as u32);

        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        cortex_m::interrupt::free(|_cs| unsafe {
            flash::flash_range_erase_and_program(addr_to_write - FLASH_BASE_ADDR, &buffer, true);
        });
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }

    Ok(())
}

pub fn log_free_bytes() {
    let free_bytes = global_allocator::ALLOCATOR.free();
    info!("Free bytes in heap: {}", free_bytes);
}