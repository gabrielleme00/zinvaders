use minifb::{Key, Window, WindowOptions};
use std::{env, fs};
use zinvaders::{Input, SoundSystem, State};

const DISPLAY_WIDTH: usize = 224;
const DISPLAY_HEIGHT: usize = 256;
const VRAM_START: u16 = 0x2400;
const VRAM_SIZE: usize = 0x1C00;

const PIXEL_ON: u32 = 0xFFFFFF;
const PIXEL_OFF: u32 = 0x000000;

const CPU_FREQUENCY_HZ: u32 = 2_000_000; // 2 MHz
const REFRESH_RATE_HZ: u32 = 60; // 60 Hz
const FRAME_CYCLES: u32 = CPU_FREQUENCY_HZ / REFRESH_RATE_HZ; // 33333 cycles/frame @ 60Hz 
const HALF_FRAME_CYCLES: u32 = FRAME_CYCLES / 2; // 16666 cycles/frame @ 60Hz

fn main() {
    let rom_path = env::args().nth(1).expect("Usage: zinvaders <rom_path>");
    let rom = fs::read(&rom_path).expect("Failed to read ROM file");

    let mut state = State::new();

    if rom_path.to_uppercase().ends_with(".COM") {
        // Setup for .COM files (CP/M standard)
        state.mmu.load_rom(&rom, 0x0100);
        state.mmu.write_byte(0x0, 0xd3);
        state.mmu.write_byte(0x5, 0xd3);
        state.mmu.write_byte(0x6, 0x01);
        state.mmu.write_byte(0x7, 0xc9);
        state.cpu.pc = 0x100;

        // Run in headless mode for .COM files (tests)
        loop {
            state.cpu.print_state(&state.mmu);
            state.cpu.step(&mut state.mmu, &mut state.ports);

            if state.cpu.halted {
                println!("CPU halted");
                break;
            }
        }
        return;
    } else {
        state.mmu.load_rom(&rom, 0);
    }

    // Create window for Space Invaders
    let mut window = Window::new(
        "Space Invaders",
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT,
        WindowOptions {
            resize: true,
            scale: minifb::Scale::X2,
            scale_mode: minifb::ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )
    .expect("Failed to create window");

    window.set_target_fps(REFRESH_RATE_HZ as usize);

    let mut buffer: Vec<u32> = vec![0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
    let mut next_interrupt = 1;
    let mut input = Input::new();

    let mut sound_system = match SoundSystem::new() {
        Ok(s) => {
            println!("Sound system initialized");
            Some(s)
        }
        Err(e) => {
            eprintln!("Warning: Failed to initialize sound: {}.", e);
            None
        }
    };

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Update input state from keyboard
        input.update(&window.get_keys());
        state.ports.port1 = input.get_port1();
        state.ports.port2 = input.get_port2();

        // Execute CPU instructions for one frame
        let mut cycles = 0;
        while cycles < FRAME_CYCLES {
            cycles += state.cpu.step(&mut state.mmu, &mut state.ports) as u32;

            if state.cpu.halted {
                return;
            }

            // Trigger interrupt at mid-frame
            if cycles >= HALF_FRAME_CYCLES && next_interrupt == 1 {
                state.cpu.interrupt(next_interrupt, &mut state.mmu);
                next_interrupt = 2;
            }
        }

        // Trigger interrupt at end of frame
        state.cpu.interrupt(next_interrupt, &mut state.mmu);
        next_interrupt = 1;

        // Update sound system
        if let Some(ref mut sound) = sound_system {
            sound.update(state.ports.port3, state.ports.port5);
        }

        // Render the display
        render_invaders(&state, &mut buffer);

        // Update window
        window
            .update_with_buffer(&buffer, DISPLAY_WIDTH, DISPLAY_HEIGHT)
            .expect("Failed to update window");
    }
}

fn render_invaders(state: &State, buffer: &mut [u32]) {
    // Each byte represents 8 vertical pixels in the original arcade orientation
    // The screen is rotated 90 degrees counter-clockwise in the cabinet

    for offset in 0..VRAM_SIZE {
        let byte = state.mmu.read_byte(VRAM_START + offset as u16);

        // Calculate screen coordinates (rotated 90 degrees counter-clockwise)
        let x = offset / 32; // 256 / 8 = 32
        let y = 255 - (offset * 8) % 256;

        // Each bit represents one pixel
        for bit in 0..8 {
            let pixel_on = (byte & (1 << bit)) != 0;

            let screen_x = x;
            let screen_y = y - bit;

            if screen_x < DISPLAY_WIDTH && screen_y < DISPLAY_HEIGHT {
                let pixel_index = screen_y * DISPLAY_WIDTH + screen_x;
                buffer[pixel_index] = if pixel_on { PIXEL_ON } else { PIXEL_OFF };
            }
        }
    }
}
