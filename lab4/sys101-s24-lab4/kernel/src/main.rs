#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
            // Add this struct definition in your screen.rs file

extern crate alloc;

mod screen;
use crate::screen::Player;
mod allocator;

use crate::screen::{screenwriter, Writer};
use alloc::boxed::Box;
use bootloader_api::config::Mapping::Dynamic;
use bootloader_api::info::MemoryRegionKind;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use core::fmt::Write;
use core::slice;
use kernel::{serial, HandlerTable};
use pc_keyboard::DecodedKey;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::PageTable;
use x86_64::VirtAddr;
const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Dynamic); // obtain physical memory offset
    config.kernel_stack_size = 256 * 1024; // 256 KiB kernel stack size
    config
};
entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let frame_info = boot_info.framebuffer.as_ref().unwrap().info();
    let framebuffer = boot_info.framebuffer.as_mut().unwrap();
    screen::init(framebuffer);

    let usable_region = boot_info
        .memory_regions
        .iter()
        .filter(|x| x.kind == MemoryRegionKind::Usable)
        .last()
        .unwrap();
    let physical_offset = boot_info.physical_memory_offset.into_option().unwrap();
    let ptr = (physical_offset + usable_region.start) as *mut u8;

    let vault = unsafe { slice::from_raw_parts_mut(ptr, 100) };
    vault[0] = 65;
    vault[1] = 66;

    //read CR3 for current page table
    let cr3 = Cr3::read().0.start_address().as_u64();

    let cr3_page = unsafe { slice::from_raw_parts_mut((cr3 + physical_offset) as *mut usize, 6) };

    let l4_table = unsafe { active_level_4_table(VirtAddr::new(physical_offset)) };

    allocator::init_heap((physical_offset + usable_region.start) as usize, HEAP_SIZE);

    HandlerTable::new()
        .keyboard(key)
        .timer(tick)
        .startup(start)
        .start();
}

use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    static ref PLAYER: Mutex<Player> = Mutex::new(Player::new(50, 50, 20, 20, (0xff, 0, 0)));
}

pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}
fn start() {
            // Assuming the Writer and screenwriter() are properly initialized
        let frame_info = screenwriter().info;
        let center_x = frame_info.width / 2;
        let center_y = frame_info.height / 2;

        // Define player size
        let player_width = 20; // Width of the player
        let player_height = 20; // Height of the player

        // Calculate top-left corner of the player
        let player_x = center_x - player_width / 2;
        let player_y = center_y - player_height / 2;

        // Create and draw the player
        let player = Player::new(
            player_x,
            player_y,
            player_width,
            player_height,
            (0xff, 0, 0),
        ); // red color
        player.draw(screenwriter());
    loop {
        // Draw the player at the new position
        let player = PLAYER.lock();
        player.draw(screenwriter());
    }
}

fn tick() {}

fn key(key: DecodedKey) {
    let mut player = PLAYER.lock();
    match key {
        DecodedKey::Unicode('w') => player.y = player.y.saturating_sub(100), // Move up
        DecodedKey::Unicode('s') => player.y += 100,                         // Move down
        DecodedKey::Unicode('a') => player.x = player.x.saturating_sub(100), // Move left
        DecodedKey::Unicode('d') => player.x += 100,                         // Move right
        _ => {}
    }
}
