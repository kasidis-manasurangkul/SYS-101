#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
            // Add this struct definition in your screen.rs file

extern crate alloc;
use alloc::vec::Vec;
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
    config.kernel_stack_size = 2560 * 1024; // 256 KiB kernel stack size
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
// row number
const ROWS: usize = 4;

const BARRIER_COLS: usize = 20;
const BARRIER_ROWS: usize = 4;
lazy_static! {
    static ref SCORE: Mutex<u32> = Mutex::new(0);
    static ref GAMEOVER: Mutex<bool> = Mutex::new(false);
    static ref WINNER: Mutex<bool> = Mutex::new(false);
    // tick counter from one to five
    static ref TICK_COUNTER1: Mutex<u32> = Mutex::new(0);
    static ref TICK_COUNTER2: Mutex<u32> = Mutex::new(0);

    static ref PLAYER: Mutex<Player> = Mutex::new(Player::new(50, 50, 40, 40, (0xff, 0, 0)));
    static ref ENEMIES: Mutex<RefCell<[[Option<Enemy>; 15];ROWS]>> = Mutex::new(RefCell::new(init_enemy_array()));
    // enemy movement direction (1 for right, -1 for left)
    static ref ENEMY_DX: Mutex<i32> = Mutex::new(1);
    static ref ENEMY_BORDER: Mutex<(usize, usize)> = Mutex::new((0, 0));
    // array of enemy bullets
    static ref ENEMY_BULLETS: Mutex<RefCell<[Option<EnemyBullet>; 10]>> = Mutex::new(RefCell::new(init_enemy_bullet_array()));
    // array of bullets
    static ref BULLETS: Mutex<RefCell<[Option<Bullet>; 10]>> = Mutex::new(RefCell::new(init_bullet_array()));
    // array of barriers
    static ref BARRIERS: Mutex<RefCell<[[Option<Barrier>; BARRIER_COLS]; BARRIER_ROWS]>> = Mutex::new(RefCell::new(init_barrier_array()));
}

pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

const ENEMY_PATTERN: [(f64, f64); 38] = [
    (2.0, 0.0),
    (8.0, 0.0),
    (3.0, 1.0),
    (7.0, 1.0),
    (2.0, 2.0),
    (3.0, 2.0),
    (4.0, 2.0),
    (5.0, 2.0),
    (6.0, 2.0),
    (7.0, 2.0),
    (8.0, 2.0),
    (1.0, 3.0),
    (2.0, 3.0),
    (4.0, 3.0),
    (5.0, 3.0),
    (6.0, 3.0),
    (8.0, 3.0),
    (9.0, 3.0),
    (0.0, 4.0),
    (1.0, 4.0),
    (2.0, 4.0),
    (3.0, 4.0),
    (4.0, 4.0),
    (5.0, 4.0),
    (6.0, 4.0),
    (7.0, 4.0),
    (8.0, 4.0),
    (9.0, 4.0),
    (10.0, 4.0),
    (1.0, 5.0),
    (2.0, 5.0),
    (3.0, 5.0),
    (4.0, 5.0),
    (5.0, 5.0),
    (6.0, 5.0),
    (7.0, 5.0),
    (8.0, 5.0),
    (9.0, 5.0),
];

const PLAYER_PATTERN: [(f64, f64); 14] = [
    (1.0, 0.0),
    (2.0, 0.0),
    (3.0, 0.0),
    (4.0, 0.0),
    (0.0, 1.0),
    (1.0, 1.0),
    (2.0, 1.0),
    (3.0, 1.0),
    (4.0, 1.0),
    (5.0, 1.0),
    (0.0, 2.0),
    (1.0, 2.0),
    (4.0, 2.0),
    (5.0, 2.0),
];

fn round_f64_to_usize(value: f64) -> usize {
    let int_part = value as isize; // Get the integer part
    let frac_part = value - int_part as f64; // Subtract to get the fractional part
    if frac_part >= 0.5 {
        (int_part + 1) as usize
    } else {
        int_part as usize
    }
}
fn draw_scaled_pattern(
    writer: &mut ScreenWriter,
    pattern: &[(f64, f64)],
    top_left_x: usize,
    top_left_y: usize,
    scale_factor: f64, // Changed to f64
    r: u8,
    g: u8,
    b: u8,
) {
    for &(offset_x, offset_y) in pattern.iter() {
        for dx in 0..(scale_factor as usize) {
            // Cast to usize for loop range
            for dy in 0..(scale_factor as usize) {
                // Cast to usize for loop range
                let x = top_left_x + round_f64_to_usize(offset_x * scale_factor) + dx;
                let y = top_left_y + round_f64_to_usize(offset_y * scale_factor) + dy;
                writer.draw_pixel(x, y, r, g, b);
            }
        }
    }
}

fn start() {
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

    let enemies_guard = ENEMIES.lock();
    let mut enemies = enemies_guard.borrow_mut();

    // Enemy and spacing dimensions
    let enemy_width = 35;
    let enemy_height = 35;
    let horizontal_spacing = 10;
    let vertical_spacing = 10;
    let enemy_color = (0, 0, 0xff);

    // Calculate the total width required for enemies including spacing
    let total_enemies_width = (enemy_width + horizontal_spacing) * 15 - horizontal_spacing;
    let start_x = (frame_info.width - total_enemies_width) / 2;

    // Draw enemies with spacing
    let mut writer = screenwriter();
    for i in 0..ROWS {
        for j in 0..15 {
            let enemy_x = start_x + j * (enemy_width + horizontal_spacing);
            let enemy_y = 50 + i * (enemy_height + vertical_spacing);
            enemies[i][j] = Some(Enemy::new(
                enemy_x,
                enemy_y,
                enemy_width,
                enemy_height,
                enemy_color,
            ));
            enemies[i][j].as_ref().unwrap().draw(&mut writer);
        }
    }

    // Barrier dimensions
    let barrier_width = 30;
    let barrier_height = 20;
    // grey
    let barrier_color = (0x80, 0x80, 0x80);
    let barrier_spacing = 20; // Updated spacing between barriers

    // Calculate the total width required for barriers including new spacing
    let total_barriers_width = (barrier_width + barrier_spacing) * BARRIER_COLS - barrier_spacing;
    let start_x = (frame_info.width - total_barriers_width) / 2;

    let barriers_guard = BARRIERS.lock();
    let mut barriers = barriers_guard.borrow_mut();

    // Draw barriers with new spacing
    for i in 0..BARRIER_ROWS {
        for j in 0..BARRIER_COLS {
            let mut barrier_x = start_x + j * (barrier_width + barrier_spacing);
            if i % 2 == 0 {
                barrier_x += 30;
            } else {
                barrier_x -= 30;
            }
            let barrier_y_offset = 200; // Increase this value to raise the barriers higher
            let barrier_y =
                frame_info.height - barrier_y_offset - i * (barrier_height + barrier_spacing);

            barriers[i][j] = Some(Barrier::new(
                barrier_x,
                barrier_y,
                barrier_width,
                barrier_height,
                barrier_color,
            ));
            barriers[i][j].as_ref().unwrap().draw(&mut writer);
        }
    }
}

fn tick() {
    if *GAMEOVER.lock() {
        let mut tick_counter2 = TICK_COUNTER2.lock();
        if *tick_counter2 > 5 {
            display_game_over();
            *tick_counter2 = 0;
        } else {
            *tick_counter2 += 1;
        }
        return;
    }
    if *WINNER.lock() {
        let mut tick_counter2 = TICK_COUNTER2.lock();
        if *tick_counter2 > 5 {
            display_winner();
            *tick_counter2 = 0;
        } else {
            *tick_counter2 += 1;
        }
        return;
    }
    display_score();
    enemy_shoot();
    // Increment the tick counter
    let mut tick_counter1 = TICK_COUNTER1.lock();
    *tick_counter1 += 1;
    if *tick_counter1 > 40 {
        if !are_enemies_remaining() {
            display_winner();
            return;
        }
        enemy_movement();

        *tick_counter1 = 0;
    } else {
        *tick_counter1 += 1;
    }
    let mut tick_counter2 = TICK_COUNTER2.lock();
    *tick_counter2 += 1;
    if *tick_counter2 > 5 {
        bullet_movement();
        enemy_bullet_movement();
        *tick_counter2 = 0;
    } else {
        *tick_counter2 += 1;
    }
}

fn key(key: DecodedKey) {
    let mut player = PLAYER.lock();
    match key {
        DecodedKey::RawKey(code) => {
            let frame_info = screenwriter().info;
            match code {
                pc_keyboard::KeyCode::ArrowLeft if player.x > 0 => {
                    // write!(Writer, "left").unwrap();
                    let mut game_over = GAMEOVER.lock();
                    let mut winner = WINNER.lock();
                    if *game_over || *winner {
                        reset_game();
                        *game_over = false;
                        *winner = false;
                        *SCORE.lock() = 0;
                    }
                    player_move_left(&mut player);
                }
                pc_keyboard::KeyCode::ArrowRight if player.x + player.width < frame_info.width => {
                    let mut game_over = GAMEOVER.lock();
                    let mut winner = WINNER.lock();
                    if *game_over || *winner {
                        reset_game();
                        *game_over = false;
                        *winner = false;
                        *SCORE.lock() = 0;
                    }
                    player_move_right(&mut player);
                }
                _ => {}
            }
        }
        DecodedKey::Unicode(character) => {
            if character == ' ' {
                // Handle space bar press
                let bullets_guard = BULLETS.lock();
                let mut bullets = bullets_guard.borrow_mut();
                // Add a new bullet if under the limit
                if bullets.iter().filter(|x| x.is_some()).count() < 10 {
                    if let Some(first_empty_slot) = bullets.iter_mut().find(|x| x.is_none()) {
                        *first_empty_slot = Some(Bullet::new(
                            player.x + player.width / 2,
                            player.y - 5,
                            5,
                            5,
                            //light blue
                            (0xad, 0xd8, 0xe6),
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
        draw_scaled_pattern(
            writer,
            &PLAYER_PATTERN,
            self.x,
            self.y,
            6.6,
            self.color.0,
            self.color.1,
            self.color.2,
        );
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

fn player_move_left(player: &mut Player) {
    let mut writer = screenwriter();
    player.erase(&mut writer, (0, 0, 0));
    if player.x >= 10 {
        player.x -= 10;
    } else {
        player.x = 0; // Prevent overflow by setting to minimum value
    }
    player.draw(&mut writer);
}

fn player_move_right(player: &mut Player) {
    let mut writer = screenwriter();
    // get screen size
    let frame_info = screenwriter().info;
    player.erase(&mut writer, (0, 0, 0));
    if player.x + player.width < frame_info.width - 10 {
        player.x += 10;
    } else {
        player.x = frame_info.width - 10 - player.width; // Prevent overflow by setting to maximum value
    }
    player.draw(&mut writer);
}

struct Bullet {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: (u8, u8, u8),
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

fn init_bullet_array() -> [Option<Bullet>; 10] {
    let mut bullets = [None, None, None, None, None, None, None, None, None, None];
    // Alternatively, you can use a loop to initialize each element to None
    for bullet in bullets.iter_mut() {
        *bullet = None;
    }
    bullets
}

fn check_collision_between_player_bullet_and_enemy(bullet: &Bullet, enemy: &Enemy) -> bool {
    let bullet_right = bullet.x + bullet.width;
    let bullet_bottom = bullet.y + bullet.height;
    let enemy_right = enemy.x + enemy.width;
    let enemy_bottom = enemy.y + enemy.height;

    !(bullet.x > enemy_right
        || bullet_right < enemy.x
        || bullet.y > enemy_bottom
        || bullet_bottom < enemy.y)
}

// check collisioin between bullet and barrier
fn check_collision_between_player_bullet_and_barrier(bullet: &Bullet, barrier: &Barrier) -> bool {
    let bullet_right = bullet.x + bullet.width;
    let bullet_bottom = bullet.y + bullet.height;
    let barrier_right = barrier.x + barrier.width;
    let barrier_bottom = barrier.y + barrier.height;

    !(bullet.x > barrier_right
        || bullet_right < barrier.x
        || bullet.y > barrier_bottom
        || bullet_bottom < barrier.y)
}

fn bullet_movement() {
    let mut writer = screenwriter();
    let bullets_guard = BULLETS.lock();
    let mut bullets = bullets_guard.borrow_mut();
    let enemies_guard = ENEMIES.lock();
    let mut enemies = enemies_guard.borrow_mut();
    let barriers_guard = BARRIERS.lock();
    let mut barriers = barriers_guard.borrow_mut();

    let mut bullets_to_remove = Vec::new();
    let mut enemies_to_remove = Vec::new();
    let mut barriers_to_remove = Vec::new();

    for (i, bullet_opt) in bullets.iter_mut().enumerate() {
        if let Some(bullet) = bullet_opt {
            bullet.erase(&mut writer, (0, 0, 0));

            // Check if bullet goes out of screen or collides
            if bullet.y <= 30 {
                bullets_to_remove.push(i); // Bullet goes out of screen
            } else {
                bullet.y -= 30; // Move bullet
                let mut hit = false;

                for (j, enemy_opt) in enemies.iter_mut().enumerate() {
                    for (k, enemy) in enemy_opt.iter_mut().enumerate() {
                        if let Some(enemy) = enemy {
                            if check_collision_between_player_bullet_and_enemy(bullet, enemy) {
                                enemy_killed();
                                enemies_to_remove.push((j, k));
                                hit = true;
                                enemy.erase(&mut writer, (0, 0, 0));
                                break;
                            }
                        }
                    }

                    if hit {
                        bullets_to_remove.push(i);
                        break;
                    }
                }

                // Check if bullet collides with barrier
                for (j, barrier_row) in barriers.iter_mut().enumerate() {
                    for (k, barrier_opt) in barrier_row.iter_mut().enumerate() {
                        if let Some(barrier) = barrier_opt {
                            if check_collision_between_player_bullet_and_barrier(bullet, barrier) {
                                barriers_to_remove.push((k, j));
                                bullet.erase(&mut writer, (0, 0, 0));
                                barrier.erase(&mut writer, (0, 0, 0));
                                bullets_to_remove.push(i);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    // Remove bullets that went off-screen or collided
    for &bullet_index in bullets_to_remove.iter().rev() {
        bullets[bullet_index] = None;
    }

    // Remove hit enemies
    for (i, j) in enemies_to_remove.iter() {
        enemies[*i][*j] = None;
    }

    // Remove hit barriers
    for (i, j) in barriers_to_remove.iter() {
        barriers[*j][*i] = None;
    }

    // Redraw remaining bullets
    for bullet_opt in bullets.iter() {
        if let Some(bullet) = bullet_opt {
            bullet.draw(&mut writer);
        }
    }

    // Redraw remaining enemies
    for enemy_row in enemies.iter_mut() {
        for enemy_opt in enemy_row {
            if let Some(enemy) = enemy_opt {
                enemy.draw(&mut writer);
            }
        }
    }

    // Redraw remaining barriers
    for barrier_row in barriers.iter_mut() {
        for barrier_opt in barrier_row {
            if let Some(barrier) = barrier_opt {
                barrier.draw(&mut writer);
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
fn init_enemy_array() -> [[Option<Enemy>; 15]; ROWS] {
    [[ARRAY_REPEAT_VALUE; 15]; ROWS] // Initialize all enemies to None
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
        draw_scaled_pattern(
            writer,
            &ENEMY_PATTERN,
            self.x,
            self.y,
            3.0,
            self.color.0,
            self.color.1,
            self.color.2,
        );
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

fn enemy_movement() {
    let enemies_guard = ENEMIES.lock();
    let mut enemies = enemies_guard.borrow_mut();
    let mut enemy_dx = ENEMY_DX.lock();
    let frame_info = screenwriter().info;

    // Find the positions of the foremost enemies
    let (first_x, last_x) = find_foremost_enemies_positions(&*enemies);

    // Determine if direction change is needed
    if first_x < 30 && *enemy_dx == -1 || last_x + 30 > frame_info.width - 30 && *enemy_dx == 1 {
        *enemy_dx *= -1; // Change direction
        move_enemies_down(&mut enemies, 100); // Move all enemies down by 50 pixels
    } else {
        // Continue moving enemies in the current horizontal direction
        let mut writer = screenwriter();
        for enemy_opt in enemies.iter_mut() {
            for enemy in enemy_opt.iter_mut() {
                if let Some(enemy) = enemy {
                    enemy.erase(&mut writer, (0, 0, 0));
                    enemy.x = (enemy.x as i32 + *enemy_dx * 15) as usize; // Move the enemy
                    enemy.draw(&mut writer);
                }
            }
        }
    }
}

fn move_enemies_down(enemies: &mut [[Option<Enemy>; 15]; ROWS], down_step: usize) {
    let mut writer = screenwriter();
    for enemy_opt in enemies.iter_mut() {
        for enemy in enemy_opt.iter_mut() {
            if let Some(enemy) = enemy {
                enemy.erase(&mut writer, (0, 0, 0));
                enemy.y += down_step; // Move the enemy down
                let frame_info = screenwriter().info;
                if enemy.y > frame_info.height - 100 - 50 {
                    let mut is_game_over = GAMEOVER.lock();
                    *is_game_over = true;
                }
                enemy.draw(&mut writer);
            }
        }
    }
}

struct EnemyBullet {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: (u8, u8, u8),
}

impl EnemyBullet {
    pub fn new(x: usize, y: usize, width: usize, height: usize, color: (u8, u8, u8)) -> Self {
        EnemyBullet {
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

fn init_enemy_bullet_array() -> [Option<EnemyBullet>; 10] {
    let mut enemy_bullets = [None, None, None, None, None, None, None, None, None, None];
    // Alternatively, you can use a loop to initialize each element to None
    for bullet in enemy_bullets.iter_mut() {
        *bullet = None;
    }
    enemy_bullets
}

fn enemy_shoot() {
    // Example: Random enemy shoots a bullet
    // This is a basic example, consider a more sophisticated approach
    let enemies_guard = ENEMIES.lock();
    let enemies = enemies_guard.borrow();
    let enemy_bullets_guard = ENEMY_BULLETS.lock();
    let mut enemy_bullets = enemy_bullets_guard.borrow_mut();

    if let Some((x, y)) = select_random_enemy_position(&*enemies) {
        // Add a new bullet if under the limit
        if enemy_bullets.iter().filter(|x| x.is_some()).count() < 10 {
            if let Some(first_empty_slot) = enemy_bullets.iter_mut().find(|x| x.is_none()) {
                *first_empty_slot = Some(EnemyBullet::new(x + 20, y + 40, 5, 5, (0, 0xff, 0)));
            }
        }
    }
}

fn select_random_enemy_position(enemies: &[[Option<Enemy>; 15]]) -> Option<(usize, usize)> {
    let mut available_enemies = Vec::new();

    for (j, enemy_opt) in enemies[0].iter().enumerate() {
        if enemy_opt.is_some() {
            available_enemies.push(j);
        }
    }

    if available_enemies.is_empty() {
        None
    } else {
        // Use a simple counter to select an enemy
        let counter = *TICK_COUNTER1.lock();
        let enemy_index = counter as usize % available_enemies.len();
        let enemy_col = available_enemies[enemy_index];
        enemies[0][enemy_col]
            .as_ref()
            .map(|enemy| (enemy.x, enemy.y))
    }
}

fn enemy_bullet_movement() {
    let mut writer = screenwriter();
    let enemy_bullets_guard = ENEMY_BULLETS.lock();
    let mut enemy_bullets = enemy_bullets_guard.borrow_mut();
    let barriers_guard = BARRIERS.lock();
    let mut barriers = barriers_guard.borrow_mut();

    let mut bullets_to_remove = Vec::new();
    let mut barriers_to_remove = Vec::new();

    let player = PLAYER.lock();

    for (i, bullet_opt) in enemy_bullets.iter_mut().enumerate() {
        if let Some(bullet) = bullet_opt {
            bullet.erase(&mut writer, (0, 0, 0));
            let mut hit = false;
            // Check for collision with player
            if check_collision_between_enemy_bullet_and_player(&bullet, &player) {
                let mut is_game_over = GAMEOVER.lock();
                *is_game_over = true;
            }

            // Check if bullet collides with barrier
            for (j, barrier_row) in barriers.iter().enumerate() {
                for (k, barrier_opt) in barrier_row.iter().enumerate() {
                    if let Some(barrier) = barrier_opt {
                        if check_collision_between_enemy_bullet_and_barrier(bullet, barrier) {
                            hit = true;
                            barriers_to_remove.push((k, j));
                            bullets_to_remove.push(i);
                            barrier.erase(&mut writer, (0, 0, 0));
                            break;
                        }
                    }
                }
            }

            // Check if bullet goes off-screen
            let frame_info = screenwriter().info;
            if bullet.y + bullet.height + 30 >= frame_info.height {
                bullets_to_remove.push(i); // Bullet goes off the bottom of the screen
            } else {
                if !hit {
                    bullet.y += 30; // Move bullet downwards

                    // Redraw the bullet at its new position
                    bullet.draw(&mut writer);
                } else {
                    bullet.erase(&mut writer, (0, 0, 0));
                }
            }
        }
    }

    // Remove bullets that went off-screen or collided
    for &bullet_index in bullets_to_remove.iter().rev() {
        enemy_bullets[bullet_index] = None;
    }

    // Remove hit barriers
    for (i, j) in barriers_to_remove.iter() {
        barriers[*j][*i] = None;
    }

    // Redraw remaining bullets
    for bullet_opt in enemy_bullets.iter() {
        if let Some(bullet) = bullet_opt {
            bullet.draw(&mut writer);
        }
    }

    // Redraw remaining barriers
    for barrier_row in barriers.iter_mut() {
        for barrier_opt in barrier_row.iter_mut() {
            if let Some(barrier) = barrier_opt {
                barrier.draw(&mut writer);
            }
        }
    }
}

fn check_collision_between_enemy_bullet_and_player(bullet: &EnemyBullet, player: &Player) -> bool {
    let bullet_right = bullet.x + bullet.width;
    let bullet_bottom = bullet.y + bullet.height;
    let player_right = player.x + player.width;
    let player_bottom = player.y + player.height;

    !(bullet.x > player_right
        || bullet_right < player.x
        || bullet.y > player_bottom
        || bullet_bottom < player.y)
}

fn check_collision_between_enemy_bullet_and_barrier(
    bullet: &EnemyBullet,
    barrier: &Barrier,
) -> bool {
    let bullet_right = bullet.x + bullet.width;
    let bullet_bottom = bullet.y + bullet.height;
    let barrier_right = barrier.x + barrier.width;
    let barrier_bottom = barrier.y + barrier.height;

    !(bullet.x > barrier_right
        || bullet_right < barrier.x
        || bullet.y > barrier_bottom
        || bullet_bottom < barrier.y)
}

fn find_foremost_enemies_positions(enemies: &[[Option<Enemy>; 15]]) -> (usize, usize) {
    let mut first_x = usize::MAX;
    let mut last_x = 0;

    for row in enemies {
        if let Some(first_enemy) = row.iter().find(|e| e.is_some()).and_then(|e| e.as_ref()) {
            first_x = first_x.min(first_enemy.x);
        }
        if let Some(last_enemy) = row
            .iter()
            .rev()
            .find(|e| e.is_some())
            .and_then(|e| e.as_ref())
        {
            last_x = last_x.max(last_enemy.x);
        }
    }

    (first_x, last_x)
}

#[derive(Copy, Clone)]
struct Barrier {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: (u8, u8, u8),
}

impl Barrier {
    pub fn new(x: usize, y: usize, width: usize, height: usize, color: (u8, u8, u8)) -> Self {
        Barrier {
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

fn init_barrier_array() -> [[Option<Barrier>; BARRIER_COLS]; BARRIER_ROWS] {
    const ARRAY_REPEAT_VALUE: Option<Barrier> = None;
    let mut barriers = [[ARRAY_REPEAT_VALUE; BARRIER_COLS]; BARRIER_ROWS];
    // Alternatively, you can use a loop to initialize each element to None
    for barrier_row in barriers.iter_mut() {
        for barrier in barrier_row.iter_mut() {
            *barrier = None;
        }
    }
    barriers
}

fn display_score() {
    let writer = screenwriter();

    // Set the cursor position for score display at the top left
    let score_display_x = 10; // A small margin from the left edge
    let score_display_y = 10; // A small margin from the top edge

    // Clear the score display area if needed, or draw over it directly

    // Set the cursor position
    writer.set_position(score_display_x, score_display_y);

    // Get and display the current score
    let score = SCORE.lock();
    let _ = write!(writer, "Score: {}", *score);
}

fn enemy_killed() {
    // Increment the score by 10 for each enemy killed
    let mut score = SCORE.lock();
    *score += 10;
}

fn display_game_over() {
    let writer = screenwriter();
    writer.clear(); // Clear the entire screen

    // set player to None
    // let mut player = PLAYER.lock();
    // player.erase(&mut writer, (0, 0, 0));
    // *player = Player::new(0, 0, 0, 0, (0, 0, 0));

    // Set the position for the Game Over message
    let message_x = writer.info.width / 2 - 40; // Adjust as needed
    let message_y = writer.info.height / 2;
    writer.set_position(message_x, message_y);
    let _ = write!(writer, "GAME OVER");
    writer.set_position(message_x - 80, message_y + 20); // Adjust Y position for next line
    let _ = write!(writer, "Move left or right to retry");

    // // Display the Retry message
    // writer.set_position(message_x - 30, message_y + 20); // Adjust Y position for next line
    // let _ = write!(writer, "Press R to Retry");
}

fn display_winner() {
    let mut winner = WINNER.lock();
    *winner = true;
    let writer = screenwriter();
    writer.clear(); // Clear the screen

    // Set the position for the winning message
    let message_x = writer.info.width / 2 - 40; // Adjust as needed
    let message_y = writer.info.height / 2;
    writer.set_position(message_x, message_y);
    let _ = write!(writer, "YOU WIN!");
    writer.set_position(message_x - 35, message_y + 20); // Adjust Y position for next line
    let _ = write!(writer, "Press R to Restart");

    // Optionally display a restart message or any other information
}

fn are_enemies_remaining() -> bool {
    let enemies_guard = ENEMIES.lock();
    let enemies = enemies_guard.borrow();

    for enemy_row in enemies.iter() {
        for enemy_opt in enemy_row {
            if enemy_opt.is_some() {
                return true; // There are still enemies remaining
            }
        }
    }
    false // No enemies remaining
}

fn reset_game() {
    let frame_info = screenwriter().info;
    // restore enemies
    let enemies_guard = ENEMIES.lock();
    let mut enemies = enemies_guard.borrow_mut();

    // Enemy and spacing dimensions
    let enemy_width = 35;
    let enemy_height = 35;
    let horizontal_spacing = 10;
    let vertical_spacing = 10;
    let enemy_color = (0, 0, 0xff);

    // Calculate the total width required for enemies including spacing
    let total_enemies_width = (enemy_width + horizontal_spacing) * 15 - horizontal_spacing;
    let start_x = (frame_info.width - total_enemies_width) / 2;

    // Draw enemies with spacing
    let mut writer = screenwriter();
    writer.clear(); // Clear the screen
    for i in 0..ROWS {
        for j in 0..15 {
            let enemy_x = start_x + j * (enemy_width + horizontal_spacing);
            let enemy_y = 50 + i * (enemy_height + vertical_spacing);
            enemies[i][j] = Some(Enemy::new(
                enemy_x,
                enemy_y,
                enemy_width,
                enemy_height,
                enemy_color,
            ));
            enemies[i][j].as_ref().unwrap().draw(&mut writer);
        }
    }

    // Barrier dimensions
    let barrier_width = 30;
    let barrier_height = 20;
    // grey
    let barrier_color = (0x80, 0x80, 0x80);
    let barrier_spacing = 20; // Updated spacing between barriers

    // Calculate the total width required for barriers including new spacing
    let total_barriers_width = (barrier_width + barrier_spacing) * BARRIER_COLS - barrier_spacing;
    let start_x = (frame_info.width - total_barriers_width) / 2;

    let barriers_guard = BARRIERS.lock();
    let mut barriers = barriers_guard.borrow_mut();

    // Draw barriers with new spacing
    for i in 0..BARRIER_ROWS {
        for j in 0..BARRIER_COLS {
            let mut barrier_x = start_x + j * (barrier_width + barrier_spacing);
            if i % 2 == 0 {
                barrier_x += 30;
            } else {
                barrier_x -= 30;
            }
            let barrier_y_offset = 200; // Increase this value to raise the barriers higher
            let barrier_y =
                frame_info.height - barrier_y_offset - i * (barrier_height + barrier_spacing);

            barriers[i][j] = Some(Barrier::new(
                barrier_x,
                barrier_y,
                barrier_width,
                barrier_height,
                barrier_color,
            ));
            barriers[i][j].as_ref().unwrap().draw(&mut writer);
        }
    }

    // remove bullets
    let bullets_guard = BULLETS.lock();
    let mut bullets = bullets_guard.borrow_mut();
    for bullet in bullets.iter_mut() {
        *bullet = None;
    }

    // remove enemy bullets
    let enemy_bullets_guard = ENEMY_BULLETS.lock();
    let mut enemy_bullets = enemy_bullets_guard.borrow_mut();
    for bullet in enemy_bullets.iter_mut() {
        *bullet = None;
    }
}
