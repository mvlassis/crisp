use chip8_core::*;
use config::Config;
use std::env;
use std::fs::File;
use std::io::Read;
use std::time::Duration;

use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::video::Window;


const SCALE:u32 = 10;
const TICKS_PER_FRAME: usize = 15;

struct Settings {
	fg_color: Color,
	bg_color: Color,
}

struct SquareWave {
	phase_inc: f32,
	phase: f32,
	volume: f32,
}

impl AudioCallback for SquareWave {
	type Channel = f32;

	fn callback(&mut self, out: &mut [f32]) {
		for x in out.iter_mut() {
			*x = if self.phase <= 0.5 {
				self.volume
			} else {
				-self.volume
			};
			self.phase = (self.phase + self.phase_inc) % 1.0;
		}
	}
}

fn main() {
	let args: Vec<_> = env::args().collect();
	if args.len() != 2 {
		println!("Usage: cargo run <path/to/game>");
		return
	}

	let config = Config::builder()
		.add_source(config::File::with_name("config.toml"))
		.build()
		.unwrap();

	let palette = config.get_string("frontend.palette").unwrap();
	let palette_name = "frontend.".to_string() + &palette;
	let palette_colors = config.get_array(&palette_name).unwrap();
	let palette_colors: Vec<String> = palette_colors.into_iter().filter_map(|value| value.into_string().ok()).collect();

	let bg_color = &palette_colors[0];
	let (r, g, b) = hex_to_rgb(&bg_color).ok_or("Wrong foreground color").expect("Foreground color");
	let bg_color = Color::RGB(r, g, b);	
	
	let fg_color = &palette_colors[1];
	let (r, g, b) = hex_to_rgb(&fg_color).ok_or("Wrong foreground color").expect("Foreground color");
	let fg_color = Color::RGB(r, g, b);
	
	
	let sdl_context = sdl2::init().unwrap();
	let video_subsystem = sdl_context.video().unwrap();

	let selected_variant = Variant::SChip;
	let display_wait = match selected_variant {
		Variant::Chip8 => true,
		_ => false,
	};
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
	let window_width = (screen_width as u32) * SCALE;
	let window_height = (screen_height as u32) * SCALE;
		
	let window = video_subsystem.window("CHIP-8 Emulator", window_width, window_height)
		.position_centered().opengl().build().unwrap();
	
	let mut canvas = window.into_canvas().present_vsync().build().unwrap();
	canvas.clear();
	canvas.present();

	let audio_subsystem = sdl_context.audio().unwrap();
	let desired_spec = AudioSpecDesired {
		freq: Some(44100),
		channels: Some(1), // mono
		samples: None
	};
	let audio_device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
		SquareWave {
			phase_inc: 440.0 / spec.freq as f32,
			phase: 0.0,
			volume: 0.25
		}
	}).unwrap();
	

	let mut event_pump = sdl_context.event_pump().unwrap();

	let mut chip8_emulator = Emulator::new(selected_variant);
	let mut rom = File::open(&args[1]).expect("Unable to open file");
	let mut buffer = Vec::new();
	rom.read_to_end(&mut buffer).unwrap();
	chip8_emulator.load(&buffer);
	
	'running: loop {
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit {..} |
				Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
					break 'running
				},
				Event::KeyDown { keycode: Some(Keycode::T), .. } => {
					chip8_emulator.reset();
					chip8_emulator.load(&buffer);
			}
				Event::KeyDown { keycode: Some(key), ..} => {
					if let Some(k) = key2button(key) {
						chip8_emulator.register_keypress(k, true);
					}
				}
				Event::KeyUp {keycode: Some(key), ..} => {
					if let Some(k) = key2button(key) {
						chip8_emulator.register_keypress(k, false);
					}
				}
				_ => ()
			}
		}
		for i in 0..TICKS_PER_FRAME {
			if display_wait {
				match i {
					0 => chip8_emulator.tick(true),
					_ => chip8_emulator.tick(false),
				}				
			}
			else {
				chip8_emulator.tick(true);				
			}
		}
		chip8_emulator.tick_timers();
		draw_window(&chip8_emulator, &mut canvas, bg_color, fg_color, screen_width);
		//audio_device.lock();
		if chip8_emulator.beep {
			audio_device.resume();
		} else {
			audio_device.pause();
		}
		::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
	}
}

fn draw_window(emu: &Emulator, canvas: &mut Canvas<Window>, bg_color: Color, fg_color: Color, screen_width: usize) {

	canvas.set_draw_color(bg_color);
	canvas.clear();

	let screen_buffer = emu.get_display();
	canvas.set_draw_color(fg_color);
	for (i, pixel) in screen_buffer.iter().enumerate() {
		if *pixel {
			let x = (i % screen_width) as u32;
			let y = (i / screen_width) as u32;

			let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
			canvas.fill_rect(rect).unwrap();
		}
	}

	canvas.present();
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

fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
	if hex.len() != 7 {
		return None;
	}

	let r = u8::from_str_radix(&hex[1..3], 16).ok()?;
	let g = u8::from_str_radix(&hex[3..5], 16).ok()?;
	let b = u8::from_str_radix(&hex[5..7], 16).ok()?;

	Some((r, g, b))
}
