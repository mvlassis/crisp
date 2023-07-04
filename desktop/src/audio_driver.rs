// We deal with all audio work through a simple struct named AudioDriver

use chip8_core::PATTERN_BUFFER_SIZE;
use sdl2::audio::{AudioCallback, AudioSpecDesired, AudioDevice};
use sdl2::AudioSubsystem;

struct SquareWave {
	buffer_pointer: usize, // Holds the current position of the buffer
	pattern_buffer: [u8; PATTERN_BUFFER_SIZE], 
}

impl AudioCallback for SquareWave {
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
	audio_device: AudioDevice<SquareWave>,

	play_sound: bool,

	pub pattern_buffer: [u8; PATTERN_BUFFER_SIZE],
	pub frequency: i32
}

impl AudioDriver {
	pub fn new(audio_subsystem: &AudioSubsystem, buffer: [u8; PATTERN_BUFFER_SIZE], desired_frequency: i32) -> Self {
		let spec = AudioSpecDesired {
			freq: Some(desired_frequency),
			channels: Some(1), // mono sound
			samples: Some(4), // the size of the audio buffer used by the SDL system
		};
		let audio = audio_subsystem.open_playback(None, &spec, |_| {
			SquareWave {
				buffer_pointer: 0,
				pattern_buffer: buffer,
			}
		}).unwrap();
		AudioDriver {
			desired_spec: spec,
			audio_device: audio,

			play_sound: false,
			
			pattern_buffer: buffer,
			frequency: desired_frequency,
		}
	}

	// Call this once per frame to play the correct sound
	pub fn handle_audio(&mut self, beep: bool) {
		if !self.play_sound && beep {
			self.play_sound = true;
			self.audio_device.resume();
		}
		else if self.play_sound && !beep {
			self.play_sound = false;
			self.audio_device.pause();
		}
	}

	// Update the pattern buffer with the one given
	pub fn update_pattern_buffer(&mut self, new_buffer: [u8; PATTERN_BUFFER_SIZE]) {
		self.pattern_buffer = new_buffer;
		let mut lock_guard = self.audio_device.lock();
		lock_guard.pattern_buffer = new_buffer;
		drop(lock_guard);
	}

	// Update the frequency with the one given
	pub fn update_frequency(&mut self, audio_subsystem: &AudioSubsystem, new_frequency: i32) {
		let lock_guard = self.audio_device.lock();
		let position = lock_guard.buffer_pointer;
		drop(lock_guard);
		self.desired_spec.freq = Some(new_frequency);
		self.audio_device.pause();
		self.audio_device = audio_subsystem.open_playback(None, &self.desired_spec, |_| {
			SquareWave {
				buffer_pointer: position,
				pattern_buffer: self.pattern_buffer,
			}
		}).unwrap();
	}
}
