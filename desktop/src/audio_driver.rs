// We deal with all audio work through a simple struct named AudioDriver

use chip8_core::PATTERN_BUFFER_SIZE;
use chip8_core::Variant;

use sdl2::audio::{AudioCallback, AudioSpecDesired, AudioDevice};
use sdl2::AudioSubsystem;


// Simple struct that represents a square wave
struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32
}

// Audio callback function used by SDL
impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
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

struct PatternWave {
	buffer_pointer: usize, // Holds the current position of the buffer
	pattern_buffer: [u8; PATTERN_BUFFER_SIZE],

}

// Audio callback function used by SDL
impl AudioCallback for PatternWave {
	type Channel = u8;

	fn callback(&mut self, out: &mut [u8]) {
		// Fills the SDL buffer with the next bites from the pattern buffer
		for i in 0..out.len() {
			// Get the next bit
			let next_byte = self.pattern_buffer[self.buffer_pointer/8];
			out[i] = next_byte;
			out[i] <<= self.buffer_pointer % 8;
			out[i] &= 0x80;
			// If the bit is 0, then change it to the volume
			if out[i] > 0 {
				out[i] = 255;
			}

			self.buffer_pointer += 1;
			// If the pattern_buffer ends, then restart from the beginning
			if self.buffer_pointer >= PATTERN_BUFFER_SIZE * 8 {
				self.buffer_pointer = 0;
			}
		}
	}
}

pub struct AudioDriver {
	desired_spec: AudioSpecDesired,
	chip8_audio_device: Option <AudioDevice<SquareWave>>,
	xo_audio_device: Option <AudioDevice<PatternWave>>,

	play_sound: bool,

	pub pattern_buffer: [u8; PATTERN_BUFFER_SIZE],
	pub frequency: i32
}

impl AudioDriver {
	pub fn new(audio_subsystem: &AudioSubsystem, variant: &Variant, buffer: [u8; PATTERN_BUFFER_SIZE], desired_frequency: i32) -> Self {
		
		let spec = if *variant == Variant::XOChip {
			AudioSpecDesired {
				freq: Some(desired_frequency),
				channels: Some(1), // mono sound
				samples: Some(4), // the size of the audio buffer used by the SDL system
			}
		} else {
			AudioSpecDesired {
				freq: Some(44100),
				channels: Some(1), // mono sound
				samples: None, 
			}
		};

		let chip8_device = if *variant == Variant::XOChip {
			None
		} else {
			let device = audio_subsystem.open_playback(None, &spec, |_| {
				SquareWave {
					phase_inc: 440.0 / spec.freq.unwrap() as f32,
					phase: 0.0,
					volume: 0.25
				}
			}).unwrap();
			device.pause();
			Some(device)
		};

		let xo_device = if *variant == Variant::XOChip {
			let device = audio_subsystem.open_playback(None, &spec, |_| {
				PatternWave {
					buffer_pointer: 0,
					pattern_buffer: buffer,
				}
			}).unwrap();
			device.pause();
			Some(device)
		} else {
			None
		};
		
		AudioDriver {
			desired_spec: spec,
			chip8_audio_device: chip8_device,
			xo_audio_device: xo_device,

			play_sound: false,
			
			pattern_buffer: buffer,
			frequency: desired_frequency,
		}
	}

	// Call this once per frame to play the correct sound
	pub fn handle_audio(&mut self, beep: bool) {
		if !self.play_sound && beep {
			println!("BEEP");
			self.play_sound = true;
			match self.chip8_audio_device {
				Some(ref device) => device.resume(),
				None => self.xo_audio_device.as_ref().unwrap().resume(),
			}
		}
		else if self.play_sound && !beep {
			println!("NO BEEP");
			self.play_sound = false;
			match self.chip8_audio_device {
				Some(ref device) => device.pause(),
				None => self.xo_audio_device.as_ref().unwrap().pause(),
			}
		}
	}

	// Update the pattern buffer with the one given
	pub fn update_pattern_buffer(&mut self, new_buffer: [u8; PATTERN_BUFFER_SIZE]) {
		 match self.xo_audio_device {
			 Some(ref mut device) => {
				 self.pattern_buffer = new_buffer;
				 let mut lock_guard = device.lock();
				 lock_guard.pattern_buffer = new_buffer;
				 drop(lock_guard);
			 }
			 None => (),
		}
	}

	// Update the frequency with the one given
	pub fn update_frequency(&mut self, audio_subsystem: &AudioSubsystem, new_frequency: i32) {
		match self.xo_audio_device {
			Some(ref mut device) => {
				let lock_guard = device.lock();
				let position = lock_guard.buffer_pointer;
				drop(lock_guard);
				self.desired_spec.freq = Some(new_frequency);
				device.pause();
				*device = audio_subsystem.open_playback(None, &self.desired_spec, |_| {
					PatternWave {
						buffer_pointer: position,
						pattern_buffer: self.pattern_buffer,
					}
				}).unwrap();
			}
			None => (),
		}
		
	}
}
