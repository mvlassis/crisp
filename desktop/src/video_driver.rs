use sdl2::VideoSubsystem;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub struct VideoConfig {
	pub scale: u32, // The amount by which we scale the display
	pub color0: Color,
	pub color1: Color,
	pub color2: Color,
	pub color3: Color,
}

pub struct VideoDriver {
	screen_width: usize,
	_screen_height: usize,
	canvas: Canvas<Window>,

	config: VideoConfig,
}

impl VideoDriver {
	pub fn new(video_subsystem: &VideoSubsystem, s_width: u32, s_height: u32, video_config: VideoConfig) -> Self {
		let new_window_width = (s_width as u32) * video_config.scale;
		let new_window_height = (s_height as u32) * video_config.scale;
		let window = video_subsystem.window("CHIP-8 Emulator", new_window_width, new_window_height).position_centered().opengl().build().unwrap();
		let mut new_canvas = window.into_canvas().present_vsync().build().unwrap();
		
		new_canvas.clear();
		new_canvas.present();
		VideoDriver {
			canvas: new_canvas,
			screen_width: s_width as usize,
			_screen_height: s_height as usize,

			config: video_config,
		}
	}

	pub fn draw_window(&mut self, buffers: (&[bool], &[bool]) ) {
		self.canvas.set_draw_color(self.config.color0);
		self.canvas.clear();

		let (screen_buffer1, screen_buffer2) = buffers;
		
		for (index, (pixel1, pixel2)) in (screen_buffer1.iter().zip(screen_buffer2.iter())).enumerate() {
			let x = (index % self.screen_width) as u32;
			let y = (index / self.screen_width) as u32;
			let rect = Rect::new((x * self.config.scale) as i32, (y * self.config.scale) as i32, self.config.scale, self.config.scale);
			if *pixel1 && *pixel2 {
				self.canvas.set_draw_color(self.config.color3);
			}
			else if *pixel1 {
				self.canvas.set_draw_color(self.config.color1);
			}
			else if *pixel2 {
				self.canvas.set_draw_color(self.config.color2);
			}
			else {
				self.canvas.set_draw_color(self.config.color0);
			}
			self.canvas.fill_rect(rect).unwrap();
		}

		self.canvas.present();
		
	}

}
