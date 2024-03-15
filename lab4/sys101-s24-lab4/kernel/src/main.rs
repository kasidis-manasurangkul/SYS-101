#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
            // Add this struct definition in your screen.rs file

extern crate alloc;

mod allocator;
mod screen;
use crate::screen::screenwriter;
use crate::screen::ScreenWriter;
use core::cell::RefCell;
use core::fmt::Write;
// use alloc::boxed::Box;
use bootloader_api::config::Mapping::Dynamic;
use bootloader_api::info::MemoryRegionKind;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
// use core::fmt::Write;
use core::slice;
use kernel::HandlerTable;
use pc_keyboard::DecodedKey;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::PageTable;
use x86_64::VirtAddr;
const HEAP_SIZE: usize = 1000 * 1024; // 100 KiB

const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Dynamic); // obtain physical memory offset
    config.kernel_stack_size = 256 * 1024; // 256 KiB kernel stack size
    config
};
entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // let frame_info = boot_info.framebuffer.as_ref().unwrap().info();
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

    let _cr3_page = unsafe { slice::from_raw_parts_mut((cr3 + physical_offset) as *mut usize, 6) };

    let _l4_table = unsafe { active_level_4_table(VirtAddr::new(physical_offset)) };

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
    // tick counter from one to five
    static ref TICK_COUNTER: Mutex<u32> = Mutex::new(0);
    static ref PLAYER: Mutex<Player> = Mutex::new(Player::new(50, 50, 40, 40, (0xff, 0, 0)));
    static ref ENEMIES: Mutex<RefCell<[[Option<Enemy>; 15];3]>> = Mutex::new(RefCell::new(init_enemy_array()));
    // array of bullets
    static ref BULLETS: Mutex<RefCell<[Option<Bullet>; 10]>> = Mutex::new(RefCell::new(init_bullet_array()));
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
    let center_y = frame_info.height - 100;

    // Define player size
    let player_width = 40; // Width of the player
    let player_height = 40; // Height of the player

    // Create and draw the player
    let mut player = PLAYER.lock();
    player.x = center_x - player_width / 2;
    player.y = center_y - player_height / 2;
    player.draw(screenwriter());

    let mut enemies_guard = ENEMIES.lock();
    let mut enemies = enemies_guard.borrow_mut();
    for i in 0..enemies.len() {
        for j in 0..enemies[i].len() {
            enemies[i][j] = Some(Enemy::new(j * 50 + 10, i * 50 + 10, 40, 40, (0, 0, 0xff)));
            enemies[i][j].as_ref().unwrap().draw(screenwriter());
        }
    }
}

fn tick() {
    // Increment the tick counter
    let mut tick_counter = TICK_COUNTER.lock();
    *tick_counter += 1;
    if *tick_counter > 5 {
        enemy_movement();
        *tick_counter = 0;
    } else {
        *tick_counter += 1;
    }
    bullet_movement();
}

fn key(key: DecodedKey) {
    // write!(screenwriter(), "{:?}", key).unwrap();
    let mut player = PLAYER.lock();
    // *player_moved = true;
    let Writer = screenwriter();
    match key {
        DecodedKey::RawKey(code) => {
            let frame_info = screenwriter().info;
            match code {
                pc_keyboard::KeyCode::ArrowLeft if player.x > 0 => {
                    // write!(Writer, "left").unwrap();
                    player_move_left(&mut player);
                }
                pc_keyboard::KeyCode::ArrowRight if player.x + player.width < frame_info.width => {
                    player_move_right(&mut player);
                }
                _ => {}
            }
        }
        DecodedKey::Unicode(character) => {
            if character == ' ' {
                // Handle space bar press
                let mut bullets_guard = BULLETS.lock();
                let mut bullets = bullets_guard.borrow_mut();
                // Add a new bullet if under the limit
                if bullets.iter().filter(|x| x.is_some()).count() < 10 {
                    if let Some(first_empty_slot) = bullets.iter_mut().find(|x| x.is_none()) {
                        *first_empty_slot = Some(Bullet::new(
                            player.x + player.width / 2,
                            player.y - 5,
                            5,
                            5,
                            (0, 0xff, 0),
                        ));
                    }
                }
            }
        }
    }
}

pub struct Player {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub color: (u8, u8, u8),
}

impl Player {
    pub fn new(x: usize, y: usize, width: usize, height: usize, color: (u8, u8, u8)) -> Self {
        Player {
            x,
            y,
            width,
            height,
            color,
        }
    }

    pub fn draw(&self, writer: &mut ScreenWriter) {
        for dx in 0..self.width {
            for dy in 0..self.height {
                writer.draw_pixel(
                    self.x + dx,
                    self.y + dy,
                    self.color.0,
                    self.color.1,
                    self.color.2,
                );
            }
        }
    }

    pub fn erase(&self, writer: &mut ScreenWriter, background_color: (u8, u8, u8)) {
        for dx in 0..self.width {
            for dy in 0..self.height {
                writer.draw_pixel(
                    self.x + dx,
                    self.y + dy,
                    background_color.0,
                    background_color.1,
                    background_color.2,
                );
            }
        }
    }
}

#[derive(Copy, Clone)]
pub struct Enemy {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub color: (u8, u8, u8),
}

const ARRAY_REPEAT_VALUE: Option<Enemy> = None;
fn init_enemy_array() -> [[Option<Enemy>; 15]; 3] {
    [[ARRAY_REPEAT_VALUE; 15]; 3] // Initialize all enemies to None
}

impl Enemy {
    pub fn new(x: usize, y: usize, width: usize, height: usize, color: (u8, u8, u8)) -> Self {
        Enemy {
            x,
            y,
            width,
            height,
            color,
        }
    }

    pub fn draw(&self, writer: &mut ScreenWriter) {
        for dx in 0..self.width {
            for dy in 0..self.height {
                writer.draw_pixel(
                    self.x + dx,
                    self.y + dy,
                    self.color.0,
                    self.color.1,
                    self.color.2,
                );
            }
        }
    }
    pub fn erase(&self, writer: &mut ScreenWriter, background_color: (u8, u8, u8)) {
        for dx in 0..self.width {
            for dy in 0..self.height {
                writer.draw_pixel(
                    self.x + dx,
                    self.y + dy,
                    background_color.0,
                    background_color.1,
                    background_color.2,
                );
            }
        }
    }
}

struct Bullet {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: (u8, u8, u8),
}

fn init_bullet_array() -> [Option<Bullet>; 10] {
    let mut bullets = [None, None, None, None, None, None, None, None, None, None];
    // Alternatively, you can use a loop to initialize each element to None
    for bullet in bullets.iter_mut() {
        *bullet = None;
    }
    bullets
}

impl Bullet {
    pub fn new(x: usize, y: usize, width: usize, height: usize, color: (u8, u8, u8)) -> Self {
        Bullet {
            x,
            y,
            width,
            height,
            color,
        }
    }

    pub fn draw(&self, writer: &mut ScreenWriter) {
        for dx in 0..self.width {
            for dy in 0..self.height {
                writer.draw_pixel(
                    self.x + dx,
                    self.y + dy,
                    self.color.0,
                    self.color.1,
                    self.color.2,
                );
            }
        }
    }
    pub fn erase(&self, writer: &mut ScreenWriter, background_color: (u8, u8, u8)) {
        for dx in 0..self.width {
            for dy in 0..self.height {
                writer.draw_pixel(
                    self.x + dx,
                    self.y + dy,
                    background_color.0,
                    background_color.1,
                    background_color.2,
                );
            }
        }
    }
}

fn redraw_player(player: &Player) {
    screenwriter().clear(); // Clear the screen
    player.draw(screenwriter()); // Draw the player at the new position
}

fn enemy_movement() {
    let mut writer = screenwriter();
    let mut enemies_guard = ENEMIES.lock();
    let mut enemies = enemies_guard.borrow_mut();
    for enemy_opt in enemies.iter_mut() {
        for enemy in enemy_opt.iter_mut() {
            if let Some(enemy) = enemy {
                enemy.erase(&mut writer, (0, 0, 0));
                enemy.x += 1;
                enemy.draw(&mut writer);
            }
        }
    }
}

fn player_move_left(player: &mut Player) {
    let mut writer = screenwriter();
    player.erase(&mut writer, (0, 0, 0));
    player.x -= 10;
    player.draw(&mut writer);
}

fn player_move_right(player: &mut Player) {
    let mut writer = screenwriter();
    player.erase(&mut writer, (0, 0, 0));
    player.x += 10;
    player.draw(&mut writer);
}

// loop through all the bullets and move them up
fn bullet_movement() {
    let mut writer = screenwriter();
    let mut bullets_guard = BULLETS.lock();
    let mut bullets = bullets_guard.borrow_mut();
    for bullet_opt in bullets.iter_mut() {
        if let Some(bullet) = bullet_opt {
            bullet.erase(&mut writer, (0, 0, 0));
            // write!(writer, "Bullet x: {}, y: {}", bullet.x, bullet.y).unwrap();
            if bullet.y > 5 {
                bullet.y -= 10;
                bullet.draw(&mut writer);
            } else {
                *bullet_opt = None; // Remove the bullet if it goes out of screen
            }
        }
    }
}
