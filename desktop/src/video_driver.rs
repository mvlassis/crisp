use config::Config;

use sdl2::VideoSubsystem;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

#[derive(Clone)]
pub struct Palette {
	pub color0: Color,
	pub color1: Color,
	pub color2: Color,
	pub color3: Color,
}

// Holds all information needed for drawing to the screen
pub struct VideoDriver {
	screen_width: usize,
	_screen_height: usize,
	canvas: Canvas<Window>,

	palettes: Vec<Palette>,
	current_palette: usize,
	scale: u32,
}

impl VideoDriver {
	pub fn new(video_subsystem: &VideoSubsystem, s_width: u32, s_height: u32, given_palettes: Vec<Palette>, given_scale: u32) -> Self {
		let new_window_width = (s_width as u32) * given_scale;
		let new_window_height = (s_height as u32) * given_scale;
		let window = video_subsystem.window("CHIP-8 Emulator", new_window_width, new_window_height).position_centered().opengl().build().unwrap();
		let mut new_canvas = window.into_canvas().present_vsync().build().unwrap();
		
		new_canvas.clear();
		new_canvas.present();
		VideoDriver {
			canvas: new_canvas,
			screen_width: s_width as usize,
			_screen_height: s_height as usize,

			palettes: given_palettes,
			current_palette: 0,
			scale: given_scale,
		}
	}

	// Draw using the 2 screen buffer and the selected palette 
	pub fn draw_window(&mut self, buffers: (&[bool], &[bool]) ) {
		self.canvas.set_draw_color(self.palettes[0].color0);
		self.canvas.clear();

		let (screen_buffer1, screen_buffer2) = buffers;
		
		for (index, (pixel1, pixel2)) in (screen_buffer1.iter().zip(screen_buffer2.iter())).enumerate() {
			let x = (index % self.screen_width) as u32;
			let y = (index / self.screen_width) as u32;
			let rect = Rect::new((x * self.scale) as i32, (y * self.scale) as i32, self.scale, self.scale);
			if *pixel1 && *pixel2 {
				self.canvas.set_draw_color(self.palettes[self.current_palette].color3);
			}
			else if *pixel1 {
				self.canvas.set_draw_color(self.palettes[self.current_palette].color1);
			}
			else if *pixel2 {
				self.canvas.set_draw_color(self.palettes[self.current_palette].color2);
			}
			else {
				self.canvas.set_draw_color(self.palettes[self.current_palette].color0);
			}
			self.canvas.fill_rect(rect).unwrap();
		}

		self.canvas.present();
	}

	// Rotate the selected palette one spot to the right
	pub fn move_palette_right(&mut self) {
		self.current_palette = (self.current_palette + 1) % self.palettes.len()
	}

	// Rotate the selected palette one spot to the left
	pub fn move_palette_left(&mut self) {
		if self.current_palette == 0 {
			self.current_palette = self.palettes.len() - 1;
		}
		else {
			self.current_palette -= 1;
		}
	}
}

// Get all palettes from config.toml. The first one is the selected one or
// the default one
pub fn get_all_palettes() -> Vec<Palette> {
	let config = Config::builder()
		.add_source(config::File::with_name("config.toml"))
		.build()
		.unwrap();

	let palette_names = config.get_array("frontend.palettes_available").unwrap();
	let palette_names: Vec<String> = palette_names.into_iter().filter_map(|value| value.into_string().ok()).collect();
	let mut all_palettes: Vec<Palette> = Vec::new();
		
	for name in &palette_names {
		let new_palette = get_palette_from_config(&config, name);
		all_palettes.push(new_palette);
	}

	// Get the selected palette, then bring it to the front of the array
	let selected_palette_name = config.get_string("frontend.palette").unwrap();
	if let Some(index) = palette_names.iter().position(|x| *x == selected_palette_name) {
		let (front, back) = all_palettes.split_at_mut(index);
		all_palettes = back.iter().chain(front.iter()).cloned().collect();
	}
	all_palettes
}

// Return a palette from the [frontend] section in a config builder
fn get_palette_from_config(config: &Config, name: &str) -> Palette {
	let palette_name = "frontend.".to_string() + name;
	let palette_colors = config.get_array(&palette_name).unwrap();
	let palette_colors: Vec<String> = palette_colors.into_iter().filter_map(|value| value.into_string().ok()).collect();

	let (r, g, b) = hex_to_rgb(&palette_colors[0]).unwrap_or((0, 0, 0));
	let color0 = Color::RGB(r, g, b);	
	
	let (r, g, b) = hex_to_rgb(&palette_colors[1]).unwrap_or((255, 255, 0));
	let color1 = Color::RGB(r, g, b);

	// If the palette has more than 2 colors then use that
	// otherwise, just plain blue
	let color2 = if palette_colors.len() > 2 {
		let (r, g, b) = hex_to_rgb(&palette_colors[2]).unwrap();
		Color::RGB(r, g, b)
	} else {
		Color::RGB(0, 0, 255)
	};

	// If the palette has more than 2 colors then use that
	// otherwise, just plain red
	let color3 = if palette_colors.len() > 3 {
		let (r, g, b) = hex_to_rgb(&palette_colors[3]).unwrap();
		Color::RGB(r, g, b)
	} else {
		Color::RGB(255, 0, 0)
	};
	
	Palette {
		color0,
		color1,
		color2,
		color3,
	}
}

// Returns a simple #XYZABC hex code to 3 integer values
fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
	if hex.len() != 7 {
		return None;
	}

	let r = u8::from_str_radix(&hex[1..3], 16).ok()?;
	let g = u8::from_str_radix(&hex[3..5], 16).ok()?;
	let b = u8::from_str_radix(&hex[5..7], 16).ok()?;

	Some((r, g, b))
}
