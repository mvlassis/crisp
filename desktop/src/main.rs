mod video_driver;
mod audio_driver;
mod cli;

use std::fs::File;
use std::io::Read;
use std::time::Duration;
// use std::time::Instant;

use clap::Parser;


use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use spin_sleep; // More accurate than thread::sleep

use audio_driver::AudioDriver;
use video_driver::VideoDriver;
use video_driver::get_all_palettes;
use chip8_core::*;

fn main() {
	let args = cli::Args::parse();

	let sdl_context = sdl2::init().unwrap();
	let video_subsystem = sdl_context.video().unwrap();

	let selected_variant = args.get_variant();
	let screen_width = match selected_variant {
		Variant::Chip8 => 64,
		Variant::SChip => 128,
		Variant::XOChip => 128,
	};
	let screen_height = match selected_variant {
		Variant::Chip8 => 32,
		Variant::SChip => 64,
		Variant::XOChip => 64,
	};			

	// Get all available palettes from the config
	let palettes = get_all_palettes();	
	let mut video_driver = VideoDriver::new(&video_subsystem, screen_width, screen_height, palettes, args.scale as u32);

	// Get settings for the emulator and create an object
	let emu_config = args.get_emuconfig();
	let mut ticks_per_frame = args.ticks_per_frame;
	let mut chip8_emulator = Emulator::new(&emu_config);
	
	let mut rom = File::open(&args.file_name).expect("Unable to open file");
	let mut data_buffer = Vec::new();
	rom.read_to_end(&mut data_buffer).unwrap();
	chip8_emulator.load(&data_buffer);

	
	let audio_subsystem = sdl_context.audio().unwrap();
	let pattern_buffer_copy = chip8_emulator.pattern_buffer.to_owned();
	let mut audio_driver = AudioDriver::new(&audio_subsystem, &selected_variant, pattern_buffer_copy, chip8_emulator.get_sound_frequency(), args.mute);

	// Used for the FPS counter
	let timer_subsystem = sdl_context.timer().unwrap();

	let mut save_state: Option<Emulator> = None;
	
	let mut event_pump = sdl_context.event_pump().unwrap();
	'running: loop {
		let start: u64 = timer_subsystem.performance_counter();
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit {..} |
				Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
					break 'running
				},
				Event::KeyDown { keycode: Some(Keycode::Backspace), .. } => {
					// Reset the emulator
					chip8_emulator.reset();
					chip8_emulator.load(&data_buffer);
				},
				Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
					// Change the palette
					video_driver.move_palette_right();
				},
				Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
					// Change the palette
					video_driver.move_palette_left();
				},
				Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
					// Increase the ticks per frame by increments of 5
					ticks_per_frame += 5;
				},
				Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
					// Decrease the ticks per frame by increments of 5
					ticks_per_frame = if ticks_per_frame <= 5 {
						5
					} else {
						ticks_per_frame - 5
					}
				},
				Event::KeyDown { keycode: Some(Keycode::M), .. } => {
					// Mute/Unmute
					audio_driver.toggle_mute();
				},
				Event::KeyDown { keycode: Some(Keycode::O), .. } => {
					// Save save
					save_state = Some(chip8_emulator.clone());
				},
				Event::KeyDown { keycode: Some(Keycode::I), .. } => {
					// If there is a save state, load it
					if save_state.is_some() {
						chip8_emulator = save_state.clone().unwrap();
					}
				},
				Event::KeyDown { keycode: Some(key), ..} => {
					if let Some(k) = key2button(key) {
						chip8_emulator.register_keypress(k, true);
					}
				},
				Event::KeyUp {keycode: Some(key), ..} => {
					if let Some(k) = key2button(key) {
						chip8_emulator.register_keypress(k, false);
					}
				},
				_ => ()
			}
		}
		for i in 0..ticks_per_frame {
			// The first tick on each frame is a "key" frame
			// This is useful for display wait 
			match i {
				0 => chip8_emulator.tick(true),
				_ => chip8_emulator.tick(false),
			}				
		}

		chip8_emulator.tick_timers();
		audio_driver.handle_audio(chip8_emulator.beep);
		
		video_driver.draw_window(chip8_emulator.get_screen_buffers());

		if chip8_emulator.get_sound_frequency() != audio_driver.frequency {
			audio_driver.update_frequency(&audio_subsystem, chip8_emulator.get_sound_frequency());
		}

		let pattern_buffer_copy = chip8_emulator.pattern_buffer.to_owned();
		if pattern_buffer_copy != audio_driver.pattern_buffer {
			audio_driver.update_pattern_buffer(pattern_buffer_copy);
		}
		
		let end: u64 = timer_subsystem.performance_counter();
		let seconds: f64 = (end - start) as f64 / timer_subsystem.performance_frequency() as f64;

		// The amount of time left to ensure 60 fps (if vsync is on)
		let nominator = 1_000_000_000u64 / 60;
		let denominator = (seconds * 1_000_000_000f64) as u64;
		let time_delay = nominator.checked_sub(denominator);
		if !args.fpscap_off {
			match time_delay {
				Some(result) => spin_sleep::sleep(Duration::new(0, result as u32)),
		
				None => (),
			}
		}
		

		// let end: u64 = timer_subsystem.performance_counter();
		// let seconds: f64 = (end - start) as f64 / timer_subsystem.performance_frequency() as f64;
		// let current_fps = 1.0 / seconds;
		// println!("FPS: {}", current_fps);
	}
}

fn key2button(key: Keycode) -> Option<usize> {
	match key {
		Keycode::Num1 => Some(0x1),
		Keycode::Num2 => Some(0x2),
		Keycode::Num3 => Some(0x3),
		Keycode::Num4 => Some(0xC),
		Keycode::Q =>    Some(0x4),
		Keycode::W =>    Some(0x5),
		Keycode::E =>    Some(0x6),
		Keycode::R =>    Some(0xD),
		Keycode::A =>    Some(0x7),
		Keycode::S =>    Some(0x8),
		Keycode::D =>    Some(0x9),
		Keycode::F =>    Some(0xE),
		Keycode::Z =>    Some(0xA),
		Keycode::X =>    Some(0x0),
		Keycode::C =>    Some(0xB),
		Keycode::V =>    Some(0xF),
		_ =>             None,
	}
}



