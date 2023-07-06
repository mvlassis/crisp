use rand::Rng;

const RAM_SIZE: usize = 4096;
const RAM_SIZE_XO: usize = 65536;

const NUM_REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

pub const PATTERN_BUFFER_SIZE: usize = 16;
const DEFAULT_PITCH: u8 = 64;

// 80 bytes for the standard font
const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
0x20, 0x60, 0x20, 0x20, 0x70, // 1
0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
0x90, 0x90, 0xF0, 0x10, 0x10, // 4
0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
0xF0, 0x10, 0x20, 0x40, 0x40, // 7
0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
0xF0, 0x90, 0xF0, 0x90, 0x90, // A
0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
0xF0, 0x80, 0x80, 0x80, 0xF0, // C
0xE0, 0x90, 0x90, 0x90, 0xE0, // D
0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

// 100 bytes for the big font
const FONTSET_BIG_SIZE: usize = 100;
const FONTSET_BIG: [u8; FONTSET_BIG_SIZE] = [
    0x3C, 0x7E, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0x7E, 0x3C, // 0
    0x18, 0x38, 0x58, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x7E, // 1
    0x3E, 0x7F, 0xC3, 0x06, 0x0C, 0x18, 0x30, 0x60, 0xFF, 0xFF, // 2
    0x3C, 0x7E, 0xC3, 0x03, 0x0E, 0x0E, 0x03, 0xC3, 0x7E, 0x3C, // 3
    0x06, 0x0E, 0x1E, 0x36, 0x66, 0xC6, 0xFF, 0xFF, 0x06, 0x06, // 4
    0xFF, 0xFF, 0xC0, 0xC0, 0xFC, 0xFE, 0x03, 0xC3, 0x7E, 0x3C, // 5
    0x3E, 0x7C, 0xC0, 0xC0, 0xFC, 0xFE, 0xC3, 0xC3, 0x7E, 0x3C, // 6
    0xFF, 0xFF, 0x03, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x60, 0x60, // 7
    0x3C, 0x7E, 0xC3, 0xC3, 0x7E, 0x7E, 0xC3, 0xC3, 0x7E, 0x3C, // 8
    0x3C, 0x7E, 0xC3, 0xC3, 0x7F, 0x3F, 0x03, 0x03, 0x3E, 0x7C, // 9
];

// Simple enum that shows what variant we should use for the emulation
#[derive(Copy, Clone, PartialEq)]
pub enum Variant {
	Chip8,
	SChip,
	XOChip,
}

// Struct that holds all information about the emulator created
#[derive(Copy, Clone)]
pub struct EmuConfig {
	pub variant: Variant,
	
	pub quirk_vfreset: bool, // 8xy1, 8xy2, 8xy3 reset register flags to 0
	pub quirk_memory: bool,
	pub quirk_displaywait: bool,
	pub quirk_clipping: bool,
	pub quirk_shifting: bool,
	pub quirk_jumping: bool,
}

#[derive(Copy, Clone, PartialEq)]
pub enum BitPlane {
	NoPlane,
	Plane1,
	Plane2,
	Both,
}

pub struct Emulator {
	config: EmuConfig,
	
	pc: u16,
	ram_size: usize,
	ram: Vec<u8>,
	screen: Vec<bool>,
	// FYI: v stands for "variable"
	v_register: [u8; NUM_REGISTERS],
	// The I register only needs 12 bits, so it's a bit overkill
	i_register: u16,
	stack_pointer: i16,
	stack: [u16; STACK_SIZE],
	keys: [bool; NUM_KEYS],
	delay_timer: u8,
	sound_timer: u8,
	pub beep: bool, // True if the system should emit sound
	screen_width: usize,
	screen_height: usize,
	key_frame: bool,

	// Needed for the SChip variant	
	high_res_mode: bool,
	rpl: [u8; 16],

	// Needed for the XOChip variant
	next_opcode_double: bool,
	screen2: Vec<bool>, // Second plane that allows us to display 2 more colors
	plane: BitPlane,
	pub pattern_buffer: [u8; PATTERN_BUFFER_SIZE],
	pub pitch: u8,
}

// Loading ROMs into RAM starts from this address
const START_ADDRESS: u16 = 0x200;


impl Emulator {
	pub fn new(given_config: &EmuConfig) -> Self {
		let width = match given_config.variant {
			Variant::Chip8 => 64,
			Variant::SChip => 128,
			Variant::XOChip => 128,
		};
		let height = match given_config.variant {
			Variant::Chip8 => 32,
			Variant::SChip => 64,
			Variant::XOChip => 64,
		};
		let platform_ram_size = match given_config.variant {
			Variant::Chip8 | Variant::SChip => RAM_SIZE,
			Variant::XOChip => RAM_SIZE_XO,
		};
		let mut new_emulator = Self {
			config: given_config.clone(),
			
			pc: START_ADDRESS,
			ram_size: platform_ram_size,
			ram: vec![0; platform_ram_size],
			screen: vec![false; width * height],
			v_register: [0; NUM_REGISTERS],
			i_register: 0,
			stack_pointer: -1,
			stack: [0; STACK_SIZE],
			keys: [false; NUM_KEYS],
			delay_timer: 0,
			sound_timer: 0,
			beep: false,
			screen_width: width,
			screen_height: height,
			key_frame: true,

			high_res_mode: false,
			rpl: [0; 16],

			next_opcode_double: false,
			screen2: vec![false; width * height],
			plane: BitPlane::Plane1,
			pattern_buffer: [0; PATTERN_BUFFER_SIZE],
			pitch: DEFAULT_PITCH,
		};

		new_emulator.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
		new_emulator.ram[FONTSET_SIZE..FONTSET_SIZE+FONTSET_BIG_SIZE].copy_from_slice(
			&FONTSET_BIG
		);
		new_emulator
	}

	// Decrement the delay and sound timers by 1
	pub fn tick_timers(&mut self) {
		if self.delay_timer > 0 {
			self.delay_timer -= 1;
		}

		if self.sound_timer > 0 {
			self.sound_timer -= 1;
			self.beep = true;
		}
		else {
			self.beep = false;
		}
	}

	// Resets all fields (you'll probably need to reload the ROM as well)
	pub fn reset(&mut self) {
		self.pc = START_ADDRESS;
		self.ram = vec![0; self.ram_size];
		self.screen = vec![false; self.screen_width * self.screen_height];
		self.v_register = [0; NUM_REGISTERS];
		self.i_register = 0;
		self.stack_pointer = -1;
		self.stack = [0; STACK_SIZE];
		self.keys = [false; NUM_KEYS];
		self.delay_timer = 0;
		self.sound_timer = 0;
		self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
		self.ram[FONTSET_SIZE..FONTSET_SIZE+FONTSET_BIG_SIZE].copy_from_slice(
			&FONTSET_BIG
		);
		self.key_frame = true;

		self.high_res_mode = false;

		self.next_opcode_double = false;
		self.screen = vec![false; self.screen_width * self.screen_height];
		self.plane = BitPlane::Plane1;
		self.pattern_buffer = [0; PATTERN_BUFFER_SIZE];
		self.pitch = DEFAULT_PITCH;
	}
	
	pub fn tick(&mut self, key_frame: bool) {
		self.key_frame = key_frame;
		// Fetch
		let op = self.fetch();
		// Decode and Execute
		self.execute(op);
	}

	// Return the 2 screen buffers
	pub fn get_screen_buffers(&self) -> (&[bool], &[bool]){
		(&self.screen, &self.screen2)
	}

	pub fn register_keypress(&mut self, index: usize, pressed: bool) {
		self.keys[index] = pressed;
	}

	pub fn load(&mut self, data: &[u8]) {
		let start = START_ADDRESS as usize;
		let end = start + data.len();
		self.ram[start..end].copy_from_slice(data);
	}

	// Push a value to the stack
	fn push(&mut self, value: u16) {
		self.stack_pointer += 1;
		self.stack[self.stack_pointer as usize] = value;
	}		

	// Pop a value from the stack
	fn pop(&mut self) -> u16 {
		self.stack_pointer -= 1;
		self.stack[(self.stack_pointer + 1) as usize]
	}
	
	fn fetch(&mut self) -> u16 {
		let higher_byte = self.ram[self.pc as usize] as u16;
		let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
		let op = (higher_byte << 8) | lower_byte;
		self.pc += 2;
		// Look at the next opcode to check if it is 4 bytes long
		// and update the next_opcode_double flag
		let higher_byte = self.ram[self.pc as usize] as u16;
		let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
		let op2 = (higher_byte << 8) | lower_byte;
		if op2 == 0xF000 {
			self.next_opcode_double = true;
		}
		else {
			self.next_opcode_double = false;
		}
		op
	}

	// Clears screen if screen = 1, screen2 if screen = 2
	fn clear_screen(&mut self, plane: &BitPlane) {
		match plane {
			BitPlane::NoPlane => (),
			BitPlane::Plane1 => {
				for pixel in self.screen.iter_mut() {
					*pixel = false;
				}
			},
			BitPlane::Plane2 => {
				for pixel in self.screen2.iter_mut() {
					*pixel = false;
				}
			},
			BitPlane::Both => {
				for pixel in self.screen.iter_mut() {
					*pixel = false;
				}
				for pixel in self.screen2.iter_mut() {
					*pixel = false;
				}
			},
		}
	}

	// Read to opcode and execute the relative command
	fn execute(&mut self, op: u16) {
		// Big-endian system, so the first we digit we read is digit1
		let digit1 = ((op & 0xF000) >> 12) as u8;
		let digit2 = ((op & 0x0F00) >> 8) as u8;
		let digit3 = ((op & 0x00F0) >> 4) as u8;
		let digit4 = (op & 0x000F) as u8;

		// println!("{:#04x}", op);

		match (digit1, digit2, digit3, digit4) {
			// 0000: Nop
			(0x0, 0x0, 0x0, 0x0) => self.opcode_0000(),
			
			// 00E0: Clear screen
			(0x0, 0x0, 0xE, 0x0) => self.opcode_00e0(),
			
			// 00EE: Return from subroutine
			(0x0, 0x0, 0xE, 0xE) => self.opcode_00ee(),
			
			// 1MMM: Jump
			(0x1, _, _, _) => self.opcode_1mmm(op & 0xFFF),
			
			// 2MMM: Call subroutine
			(0x2, _, _, _) => self.opcode_2mmm(op & 0xFFF),
			
			// 3XNN: Skip if V[x] == NN
			(0x3, _, _, _) => self.opcode_3xnn(digit2, op & 0xFF),
			
			// 4XNN: Skip if V[x] != NN
			(0x4, _, _, _) => self.opcode_4xnn(digit2, op & 0xFF),
			
			// 5XY0: Skip if V[x] == V[y]
			(0x5, _, _, 0) => self.opcode_5xy0(digit2, digit3),
			
			// 6XNN: V[x] = NN
			(0x6, _, _, _) => self.opcode_6xnn(digit2, op & 0xFF),
				
			// 7XNN: V[x] = V[x] + NN
			(0x7, _, _, _) => self.opcode_7xnn(digit2, op & 0xFF),
			
			// 8XY0: V[x] = V[y]
			(0x8, _, _, 0) => self.opcode_8xy0(digit2, digit3),
			
			// 8YX1; V[x] = V[x] OR V[y]
			(0x8, _, _, 1) => self.opcode_8xy1(digit2, digit3),
			
			// 8XY2: V[x] = V[x] AND V[y]
			(0x8, _, _, 2) => self.opcode_8xy2(digit2, digit3),

			// 8XY3: V[x] = V[x] XOR V[y]
			(0x8, _, _, 3) => self.opcode_8xy3(digit2, digit3),
			
			// 8XY4: V[x] = V[x] + V[y], V[F] = 1 if overflow
			(0x8, _, _, 4) => self.opcode_8xy4(digit2, digit3),
			
			// 8XY5: V[x] = V[x] - V[y], V[0xF] = 1 if overflow
			(0x8, _, _, 5) => self.opcode_8xy5(digit2, digit3),
			
			// 8XY6: V[x] = V[x] >> 1
			(0x8, _, _, 6) => self.opcode_8xy6(digit2, digit3),
			
			// 8XY7: V[x] = V[y] - V[x]
			(0x8, _, _, 7) => self.opcode_8xy7(digit2, digit3),
			
			// 8XYE: V[x] = V[x] << 1
			(0x8, _, _, 0xE) => self.opcode_8xye(digit2, digit3),
			
			// 9XY0: Skip if V[x] != V[y]
			(0x9, _, _, 0) => self.opcode_9xy0(digit2, digit3),
			
			// AMMM: I = MMM
			(0xA, _, _, _) => self.opcode_ammm(op & 0xFFF),
			
			// BMMM: Jump to MMM + V[0]
			(0xB, _, _, _) => self.opcode_bmmm(op & 0xFFF, digit2),
			
			// CXNN: Get random byte, then AND with NN
			(0xC, _, _, _) => self.opcode_cxnn(digit2, op & 0xFF),
			
			// DXYN: Draw sprite of N rows at coordinates V[x], V[y]
			(0xD, _, _, _) => self.opcode_dxyn(digit2, digit3, digit4),
			
			// EX9E - Skip if key VX is pressed
			(0xE, _, 0x9, 0xE) => self.opcode_ex9e(digit2),

			// EXA1 - Skip if key VX is not pressed
			(0xE, _, 0xA, 0x1) => self.opcode_exa1(digit2),
			
			// FX07 - VX = Time: Get current timer value
			(0xF, _, 0x0, 0x7) => self.opcode_fx07(digit2),
			
			// FX0A - Wait for key press
			(0xF, _, 0x0, 0xA) => self.opcode_fx0a(digit2),
			
			// FX15: Initialize delay timer
			(0xF, _, 0x1, 0x5) => self.opcode_fx15(digit2),
			
			// FX18: Initialize sound timer
			(0xF, _, 0x1, 0x8) => self.opcode_fx18(digit2),
			
			// FX1E: Add V[x] to the memory pointer I
			(0xF, _, 0x1, 0xE) => self.opcode_fx1e(digit2),
			
			// FX29: Set I to show digit V[x]
			(0xF, _, 0x2, 0x9) => self.opcode_fx29(digit2),
			
			// FX33: Store 3 digits of V[x] at M[I]
			(0xF, _, 3, 3) => self.opcode_fx33(digit2),
			
			// FX55: Store V[0] to V[x] at M[I]
			(0xF, _, 5, 5) => self.opcode_fx55(digit2),
			
			// FX65: Load V[0] to V[x] from M[I]
			(0xF, _, 6, 5) => self.opcode_fx65(digit2),
			
			// Opcodes introduced for the SCHIP variant
			// 00FD: Exit interpreter
			(0x0, 0x0, 0xF, 0xD) => self.opcode_00fd(),
			
			// 00FE: Disable high-resolution mode
			(0x0, 0x0, 0xF, 0xE) => self.opcode_00fe(),
		
			// 00FF: Enable high-resolution mode
			(0x0, 0x0, 0xF, 0xF) => self.opcode_00ff(),
		
			// FX75: Store V[0]-V[X] in RPL flags
			(0xF, _, 0x7, 0x5) => self.opcode_fx75(digit2),
			
			// FX85: Read V[0]-V[X] from RPL flags
			// SCHIP: V <= 7, XOChip: V <= 15
			(0xF, _, 0x8, 0x5) => self.opcode_fx85(digit2),
			
			// 00CN: Scroll display N pixels down (N/2 in low resolution mode)
			(0x0, 0x0, 0xC, _) => self.opcode_00cn(digit4),
			
			// 00FB: Scroll display right by 4 pixels (2 in low resolution mode)
			(0x0, 0x0, 0xF, 0xB) => self.opcode_00fb(),
			
			// 00FC: Scroll display left by 4 pixels (2 in low resolution mode)
			(0x0, 0x0, 0xF, 0xC) => self.opcode_00fc(),
			
			// FX30: Set I to 10-byte font for digit V[x] 
			(0xF, _, 0x3, 0x0) => self.opcode_fx30(digit2),
			
			// Opcodes for the XOChip
			// 00DN: Scroll display up by N pixels (N/2 in low resolution mode)
			(0x0, 0x0, 0xD, _) => self.opcode_00dn(digit4),
			
			// 5YX2: Save V[x] to V[y] in memory starting at I
			(0x5, _, _, 0x2) => self.opcode_5xy2(digit2, digit3),
			
			// 5YX3: Load V[x] to V[y] from memory starting at I
			(0x5, _, _, 0x3) => self.opcode_5xy3(digit2, digit3),
			
			// F000: Save the next 16 bits to I
			// NOTE: This command reads 2 opcodes, so we must increment the PC again
			(0xF, 0x0, 0x0, 0x0) => self.opcode_f000(),
			
			// FN01: Select drawing plane(s)
			(0xF, _, 0x0, 0x1) => self.opcode_fn01(digit2),
			
			// F002: Store 16 bytes in audio pattern buffer
			(0xF, 0x0, 0x0, 0x2) => self.opcode_f002(),
			// FX3A: Set the pitch register to V[x]
			(0xF, _, 0x3, 0xA) => self.opcode_fx3a(digit2),
			
			(_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
		}
	}

	// Draws sprites on the screen by writing bits on the screen buffers
	fn draw_sprite(&mut self, x_base: u16, y_base: u16, n: u8, base_address: u16, plane: BitPlane) {
		let screen = if plane == BitPlane::Plane1 {
			&mut self.screen
		} else {
			&mut self.screen2
		};
		let mut flipped = 0;
		let width = if n == 0 {
			2
		} else {
			1
		};
		let mask = if n == 0 {
			0b1000_0000_0000_0000
		} else {
			0b1000_0000
		};
		let num_rows = if n == 0 {16} else {n};
		for row in 0..num_rows {
			let mut row_flipped = false;
			let address = (base_address + width*row as u16) as usize;
			let pixels = if width == 2 {
				((self.ram[address] as u16) << 8) + self.ram[address + 1] as u16
			} else {
				self.ram[address] as u16
			};
			for column in 0..(width * 8) {
				if (pixels & (mask >> column)) != 0 {
					if self.high_res_mode == true {
						if self.config.quirk_clipping {
							let x = (x_base + column as u16) as usize;
							let y = (y_base + row as u16) as usize;
							let index = x + self.screen_width * y;
							if x < self.screen_width && y < self.screen_height {
					 			row_flipped |= screen[index];
								screen[index] ^= true;
							}
						}
						else {
							let x = (x_base + column as u16) as usize % self.screen_width;
							let y = (y_base + row as u16) as usize % self.screen_height;
							let index = x + self.screen_width * y;
					 		row_flipped |= screen[index];
							screen[index] ^= true;
						}
					}
				
					else if self.high_res_mode == false {
						match self.config.variant {
							Variant::Chip8 => {
								if self.config.quirk_clipping {
									let x = (x_base + column) as usize;
									let y = (y_base + row as u16) as usize;

									let index = x + self.screen_width * y;

									if x < self.screen_width && y < self.screen_height {
										row_flipped |= screen[index];
										screen[index] ^= true;	
									}
								}
								else {
									let x = (x_base + column as u16) as usize % self.screen_width;
									let y = (y_base + row as u16) as usize % self.screen_height;
									let index = x + self.screen_width * y;
					 				row_flipped |= screen[index];
									screen[index] ^= true;
								}
							}
							Variant::SChip | Variant::XOChip => {
								// 	Get the x_base and y_base again
								// since we need to wrap around the low_res
								// low_res dimensions
								let x_base = x_base % ((self.screen_width / 2) as u16);
								let y_base = y_base % ((self.screen_height / 2) as u16);
								if self.config.quirk_clipping {
									let x = 2 * (x_base + column as u16) as usize;
									let y = 2 * (y_base + row as u16) as usize;
									let index = x + self.screen_width * y;
									if x < self.screen_width && y < self.screen_height {
										row_flipped |= screen[index];
										screen[index] ^= true;
										row_flipped |= screen[index+1];
										screen[index+1] ^= true;
										row_flipped |= screen[index+self.screen_width];
										screen[index+self.screen_width] ^= true;
										row_flipped |= screen[index+self.screen_width + 1];
										screen[index+self.screen_width+1] ^= true;
									}		
								}
								else {
									let x = 2 * (x_base + column as u16) as usize % self.screen_width;
									let y = 2 * (y_base + row as u16) as usize % self.screen_height;
									let index = x + self.screen_width * y;
									row_flipped |= screen[index];
									screen[index] ^= true;
									row_flipped |= screen[index+1];
									screen[index+1] ^= true;
									row_flipped |= screen[index+self.screen_width];
									screen[index+self.screen_width] ^= true;
									row_flipped |= screen[index+self.screen_width + 1];
									screen[index+self.screen_width+1] ^= true;
								}
							}
						}
					}
				}
			}
			if row_flipped {
				flipped += 1;
			}
		}
		if self.high_res_mode == true {
			self.v_register[0xF] = flipped;
		}
		else {
			if flipped > 0 {
				self.v_register[0xF] = 1;	
			}
			else {
				self.v_register[0xF] = 0;
			}
		}
	}

	// Helper function to scroll up a given plane
	fn scroll_up(&mut self, n: u8, plane: BitPlane) {
		let screen = if plane == BitPlane::Plane1 {
			&mut self.screen
		} else {
			&mut self.screen2
		};
		let scroll_value = if self.high_res_mode == false && self.config.variant == Variant::XOChip {
			2*n as usize
		} else {
			n as usize
		};
		for x in 0..self.screen_width {
			for y in 0..self.screen_height {
				if screen[y * self.screen_width + x] == true
					&& y >= scroll_value {
						screen[(y-scroll_value) * self.screen_width + x] = true;
					}
				screen[y * self.screen_width + x] = false;
			}
		}
	}
	
	// Helper function to scroll down a given plane
	fn scroll_down(&mut self, n: u8, plane: BitPlane) {
		let screen = if plane == BitPlane::Plane1 {
			&mut self.screen
		} else {
			&mut self.screen2
		};
		let scroll_value = if self.high_res_mode == false && self.config.variant == Variant::XOChip {
			2*n as usize
		} else {
			n as usize
		};
		for x in 0..self.screen_width {
			for y in (0..self.screen_height).rev() {
				if screen[y * self.screen_width + x] == true
					&& y < self.screen_height - scroll_value {
						screen[(y+scroll_value) * self.screen_width + x] = true;
					}
				screen[y * self.screen_width + x] = false;
			}
		}
	}
	// Helper function to scroll right a given plane
	fn scroll_right(&mut self, plane: BitPlane) {
		let screen = if plane == BitPlane::Plane1 {
			&mut self.screen
		} else {
			&mut self.screen2
		};
		let scroll_value = if self.high_res_mode == false && self.config.variant == Variant::XOChip {
			8 as usize
		} else {
			4 as usize
		};
		for y in 0..self.screen_height {
			for x in (0..self.screen_width).rev() {
				if screen[y * self.screen_width + x] == true
					&& x <	self.screen_width - scroll_value {
						screen[y * self.screen_width+x+scroll_value] = true;
					}
				screen[y * self.screen_width + x] = false;	
			}
		}
	}
	
	// Helper function to scroll left a given plane
	fn scroll_left(&mut self, plane: BitPlane) {
		let screen = if plane == BitPlane::Plane1 {
			&mut self.screen
		} else {
			&mut self.screen2
		};
		let scroll_value = if self.high_res_mode == false && self.config.variant == Variant::XOChip {
			8 as usize
		} else {
			4 as usize
		};
		for y in 0..self.screen_height {
			for x in 0..self.screen_width {
				if screen[y * self.screen_width + x] == true
					&& x >= scroll_value {
						screen[y * self.screen_width+x-scroll_value] = true;
					}
				screen[y * self.screen_width + x] = false;
			}
		}				
	}

	// Convert the pitch register to the actual frequency we will use for audio
	pub fn get_sound_frequency(&self) -> i32 {
		return (4000.0 * 2_i32.pow(((self.pitch - 64)/48) as u32) as f64) as i32;
	}
	
	// 0000: Nop
	fn opcode_0000(&self) {
		return;
	}

	// 00E0: Clear screen
	fn opcode_00e0(&mut self) {
		let current_plane = self.plane;
		self.clear_screen(&current_plane);
	}
	
	// 00EE: Return from subroutine
	fn opcode_00ee(&mut self) {
		let ret_addr = self.pop();
		self.pc = ret_addr;
	}

	// 1MMM: Jump to MMM
	fn opcode_1mmm(&mut self, nnn: u16)  {
		let next_instruction = nnn;
		self.pc = next_instruction;
	}

	// 2MMM Call subroutine NNN
	fn opcode_2mmm(&mut self, nnn: u16) {
		let next_instruction = nnn;
		self.push(self.pc);
		self.pc = next_instruction;				
	}

	// 3XNN: Skip if V[x] == NN
	fn opcode_3xnn(&mut self, x: u8, nn: u16) {
		let v_number = x as usize;
		let number_to_compare = nn as u8;
		if self.v_register[v_number] == number_to_compare {
			self.pc += 2;
			if self.next_opcode_double {
				self.pc += 2;
			}
		}
	}

	// 4XNN: Skip if V[x] != NN
	fn opcode_4xnn(&mut self, x: u8, nn: u16) {
		let v_index = x as usize;
		let number_to_compare = nn as u8;
		if self.v_register[v_index] != number_to_compare {
			self.pc += 2;
			if self.next_opcode_double {
				self.pc += 2;
			}
		}
	}

	// 5XY0: Skip if V[x] == V[y]
	fn opcode_5xy0(&mut self, x: u8, y: u8) {
		let v_index1 = x as usize;
		let v_index2 = y as usize;
		if self.v_register[v_index1] == self.v_register[v_index2] {
			self.pc += 2;
			if self.next_opcode_double {
				self.pc += 2;
			}
		}
	}

	// 6XNN: V[x] = NN
	fn opcode_6xnn(&mut self, x: u8, nn: u16) {
		let v_index = x as usize;
		let value = nn as u8;
		self.v_register[v_index] = value;
	}

	// 7XNN: V[x] = V[x] + NN
	fn opcode_7xnn(&mut self, x: u8, nn: u16) {
		let v_index = x as usize;
		let value = nn as u8;
		self.v_register[v_index] = self.v_register[v_index].wrapping_add(value);
	}

	// 8XY0: V[x] = V[y]
	fn opcode_8xy0(&mut self, x: u8, y: u8) {
		let v_index1 = x as usize;
		let v_index2 = y as usize;
		self.v_register[v_index1] = self.v_register[v_index2];
	}

	// 8YX1: V[x] = V[x] OR V[y]
	fn opcode_8xy1(&mut self, x: u8, y: u8) {
		let v_index1 = x as usize;
		let v_index2 = y as usize;
		self.v_register[v_index1] = self.v_register[v_index1] | self.v_register[v_index2];
		if self.config.quirk_vfreset {
			self.v_register[0xF] = 0;
		}
	}

	// 8XY2: V[x] = V[x] AND V[y]
	fn opcode_8xy2(&mut self, x: u8, y: u8) {
		let v_index1 = x as usize;
		let v_index2 = y as usize;
		self.v_register[v_index1] = self.v_register[v_index1] & self.v_register[v_index2];
		if self.config.quirk_vfreset {
			self.v_register[0xF] = 0;
		}
	}

	// 8XY3: V[x] = V[x] XOR V[y]
	fn opcode_8xy3(&mut self, x: u8, y: u8) {
		let v_index1 = x as usize;
		let v_index2 = y as usize;
		self.v_register[v_index1] = self.v_register[v_index1] ^ self.v_register[v_index2];
		if self.config.quirk_vfreset {
			self.v_register[0xF] = 0;
		}
	}

	// 8XY4: V[x] = V[x] + V[y], V[F] = 1 if overflow
	fn opcode_8xy4(&mut self, x: u8, y: u8) {
		let v_index1 = x as usize;
		let v_index2 = y as usize;
		let (sum, flag) = self.v_register[v_index1].overflowing_add(self.v_register[v_index2]);
		self.v_register[v_index1] = sum;
		if flag == true {
			self.v_register[0xF] = 1
		}
		else {
			self.v_register[0xF] = 0;
		}
	}
	
	// 8XY5: V[x] = V[x] - V[y], V[0xF] = 1 if overflow
	fn opcode_8xy5(&mut self, x: u8, y:u8) {
		let v_index1 = x as usize;
		let v_index2 = y as usize;
		let (dif, flag) = self.v_register[v_index1].overflowing_sub(self.v_register[v_index2]);
		self.v_register[v_index1] = dif;
		if flag == true {
			self.v_register[0xF] = 0
		}
		else {
			self.v_register[0xF] = 1;
		}
	}

	// 8XY6: V[x] = V[x] >> 1
	fn opcode_8xy6(&mut self, x: u8, y: u8) {
		let index1 = x as usize;
		let index2 = y as usize;
		if !self.config.quirk_shifting {
			self.v_register[index1] = self.v_register[index2];
		}
		let lsb = self.v_register[index1] & 1;
		self.v_register[index1] >>= 1;
		self.v_register[0xF] = lsb;
	}
	
	// 8XY7: V[x] = V[y] - V[x]
	fn opcode_8xy7(&mut self, x: u8, y: u8) {
		let v_index1 = x as usize;
		let v_index2 = y as usize;
		let (dif, flag) = self.v_register[v_index2].overflowing_sub(self.v_register[v_index1]);
		self.v_register[v_index1] = dif;
		if flag == true {
			self.v_register[0xF] = 0
		}
		else {
			self.v_register[0xF] = 1;
		}
	}
	
	// 8XYE: V[x] = V[x] << 1
	fn opcode_8xye(&mut self, x: u8, y: u8) {
		let index1 = x as usize;
		let index2 = y as usize;
		if !self.config.quirk_shifting {
			self.v_register[index1] = self.v_register[index2];
		}
		let msb = (self.v_register[index1] >> 7) & 1;
		self.v_register[index1] <<= 1;
		self.v_register[0xF] = msb;
	}	

	// 9XY0: Skip if V[x] != V[y]
	fn opcode_9xy0(&mut self, x: u8, y: u8) {
		let v_index1 = x as usize;
		let v_index2 = y as usize;
		if self.v_register[v_index1] != self.v_register[v_index2] {
			self.pc += 2;
			if self.next_opcode_double {
				self.pc += 2;
			}
		}
	}
	
	// AMMM: I = MMM
	fn opcode_ammm(&mut self, mmm: u16) {
		let index = mmm;
		self.i_register = index;
	}
	
	// BMMM: Jump to MMM + V[0]
	fn opcode_bmmm(&mut self, mmm:u16, digit2: u8) {
		let address = mmm;
		let index = if self.config.quirk_jumping {
			digit2 as usize
		} else {
			0
		};
		self.pc = address + (self.v_register[index] as u16);
	}

	// CXNN: Get random byte, then AND with NN
	fn opcode_cxnn(&mut self, x: u8, nn: u16) {
		let random_byte = rand::thread_rng().gen_range(0..=255);
		let index = x as usize;
		let value = nn as u8;
		self.v_register[index] = value & random_byte;
	}

	// DXYN: Draw sprite of N rows at coordinates V[x], V[y]
	fn opcode_dxyn(&mut self, x: u8, y: u8, n: u8) {
		let x_base = (self.v_register[x as usize] %
					  self.screen_width as u8) as u16;
		let y_base = (self.v_register[y as usize] %
					  self.screen_height as u8) as u16;
		
		match self.config.variant {
			Variant::Chip8 | Variant::SChip => {
				if self.config.quirk_displaywait {
					if self.key_frame == false {
						self.pc -= 2;
						return;
					}
				}
				let base_address = self.i_register;
				self.draw_sprite(x_base, y_base, n, base_address, BitPlane::Plane1);
			}
			
			Variant::XOChip => {
				if self.config.quirk_displaywait {
					if self.key_frame == false {
						self.pc -= 2;
						return;
					}
				}
				let mut base_address = self.i_register;
				match self.plane {
					BitPlane::NoPlane => (),
					BitPlane::Plane1 => {
						self.draw_sprite(x_base, y_base, n, base_address, BitPlane::Plane1);
					}
					BitPlane::Plane2 => {
						self.draw_sprite(x_base, y_base, n, base_address, BitPlane::Plane2);
					}
					BitPlane::Both => {
						self.draw_sprite(x_base, y_base, n, base_address, BitPlane::Plane1);
						let width = if n == 0 && self.high_res_mode == true {
							2
						} else {
							1
						};
						let num_rows = if n == 0 {16} else {n};
						// Move the address by the total number of bytes we used
						base_address = self.i_register + (width*num_rows as u16);
						self.draw_sprite(x_base, y_base, n, base_address, BitPlane::Plane2);
					}
				}
			}
		}
	}


	
	// EX9E - Skip if key VX is pressed
	fn opcode_ex9e(&mut self, x: u8) {
		// EX9E - Skip if key VX is pressed
		let index = x as usize;
		let vx = self.v_register[index];
		let key = self.keys[vx as usize];
		if key {
			self.pc += 2;
			if self.next_opcode_double {
				self.pc += 2;
			}
		}	
	}

	// EXA1 - Skip if key VX is not pressed
	fn opcode_exa1(&mut self, x: u8) {
		let index = x as usize;
		let vx = self.v_register[index];
		let key = self.keys[vx as usize];
		if !key {
			self.pc += 2;
			if self.next_opcode_double {
				self.pc += 2;
			}
		}
	}

	// FX07 - VX = Time: Get current timer value
	fn opcode_fx07(&mut self, x: u8) {
		let index = x as usize;
		self.v_register[index] = self.delay_timer;
	}
	// FX0A - Wait for key press
	fn opcode_fx0a(&mut self, x: u8) {
		let mut pressed = false;
		let index = x as usize;
		for i in 0..self.keys.len() {
			if self.keys[i] {
				self.v_register[index] = i as u8;
				pressed = true;
				break;
			}
		}
		// If no key was pressed, rewind the PC
		if !pressed {
			self.pc -= 2;
		}
	}

	// FX15: Initialize delay timer
	fn opcode_fx15(&mut self, x: u8) {
		let index = x as usize;
		self.delay_timer = self.v_register[index];
	}
	// FX18: Initialize sound timer
	fn opcode_fx18(&mut self, x: u8) {
		let index = x as usize;
		self.sound_timer = self.v_register[index];
	}
	// FX1E: Add V[x] to the memory pointer I
	fn opcode_fx1e(&mut self, x: u8) {
		let index = x as usize;
		self.i_register = self.i_register.wrapping_add(self.v_register[index] as u16);
	}
	// FX29: Set I to show digit V[x]
	fn opcode_fx29(&mut self, x: u8) {
		let index = x as usize;
		let character = self.v_register[index] as u16;
		self.i_register = character * 5;
		
	}
	// FX33: Store 3 digits of V[x] at M[I]
	fn opcode_fx33(&mut self, x: u8) {
		let index = x as usize;
		let value = self.v_register[index];
		let ones = value % 10;
		let tens = (value / 10) % 10;
		let hundreds = value / 100;
		self.ram[self.i_register as usize] = hundreds;
		self.ram[(self.i_register+1) as usize] = tens;
		self.ram[(self.i_register+2) as usize] = ones;
	}

	// FX55: Store V[0] to V[x] at M[I]
	fn opcode_fx55(&mut self, x: u8) {
		let last_index = x as usize;
		for i in 0..=last_index {
			let ram_index = (self.i_register + i as u16) as usize;
			self.ram[ram_index] = self.v_register[i];
		}
		if self.config.quirk_memory {
			self.i_register = self.i_register + last_index as u16 + 1;
		}
	}
	// FX65: Load V[0] to V[x] from M[I]
	fn opcode_fx65(&mut self, x: u8) {
		let last_index = x as usize;
		for i in 0..=last_index {
			let ram_index = (self.i_register + i as u16) as usize;
			self.v_register[i] = self.ram[ram_index];
		}
		match self.config.variant {
			Variant::Chip8 => {
				self.i_register = self.i_register + last_index as u16 + 1;		
			}
			Variant::XOChip => {
				self.i_register = self.i_register + last_index as u16 + 1;		
			}
			Variant::SChip => ()
				
		}								
	}

	// Opcodes introduced for the SCHIP variant
	// 00FD: Exit interpreter
	fn opcode_00fd(&mut self) {
		self.pc = 0x200;
	}
	// 00FE: Disable high-resolution mode
	fn opcode_00fe(&mut self) {
		self.high_res_mode = false;
		if self.config.variant == Variant::XOChip {
			self.clear_screen(&BitPlane::Both);
		}
		
	}
	// 00FF: Enable high-resolution mode
	fn opcode_00ff(&mut self) {
		self.high_res_mode = true;
		if self.config.variant == Variant::XOChip {
			self.clear_screen(&BitPlane::Both);	
		}
		
	}
	// FX75: Store V[0]-V[X] in RPL flags
	fn opcode_fx75(&mut self, x: u8) {
		let last_index = x as usize;
		self.rpl[..=last_index].copy_from_slice(&self.v_register[..=last_index]);
	}
	// FX85: Read V[0]-V[X] from RPL flags
	// SCHIP: V <= 7, XOChip: V <= 15
	fn opcode_fx85(&mut self, x: u8) {
		let last_index = x as usize;
		self.v_register[..=last_index].copy_from_slice(&self.rpl[..=last_index]);
	}

	// 00CN: Scroll display N pixels down (N/2 in low resolution mode)
	fn opcode_00cn(&mut self, n: u8) {
		match self.plane {
			BitPlane::NoPlane => (),
			BitPlane::Plane1 => {
				self.scroll_down(n, BitPlane::Plane1);
			}
			BitPlane::Plane2 => {
				self.scroll_down(n, BitPlane::Plane2);
			}
			BitPlane::Both => {
				self.scroll_down(n, BitPlane::Plane1);
				self.scroll_down(n, BitPlane::Plane2);
			}
		}
	}
	// 00FB: Scroll display right by 4 pixels (2 in low resolution mode)
	fn opcode_00fb(&mut self) {
		match self.plane {
			BitPlane::NoPlane => (),
			BitPlane::Plane1 => {
				self.scroll_right(BitPlane::Plane1);
			}
			BitPlane::Plane2 => {
				self.scroll_right(BitPlane::Plane2);
			}
			BitPlane::Both => {
				self.scroll_right(BitPlane::Plane1);
				self.scroll_right(BitPlane::Plane2);
			}
		}
	}
	// 00FC: Scroll display left by 4 pixels (2 in low resolution mode)
	fn opcode_00fc(&mut self) {
		match self.plane {
			BitPlane::NoPlane => (),
			BitPlane::Plane1 => {
				self.scroll_left(BitPlane::Plane1);
			}
			BitPlane::Plane2 => {
				self.scroll_left(BitPlane::Plane2);
			}
			BitPlane::Both => {
				self.scroll_left(BitPlane::Plane1);
				self.scroll_left(BitPlane::Plane2);
			}
		}				
	}
	// FX30: Set I to 10-byte font for digit V[x] 
	fn opcode_fx30(&mut self, x: u8) {
		let index = x as usize;
		let character = self.v_register[index] as u16;
		self.i_register = FONTSET_SIZE as u16 + character * 10;
	}

	
	// Opcodes for the XOChip
	// 00DN: Scroll display up by N pixels (N/2 in low resolution mode)
	fn opcode_00dn(&mut self, n: u8) {
		match self.plane {
			BitPlane::NoPlane => (),
			BitPlane::Plane1 => {
				self.scroll_up(n, BitPlane::Plane1);
			}
			BitPlane::Plane2 => {
				self.scroll_up(n, BitPlane::Plane2);
			}
			BitPlane::Both => {
				self.scroll_up(n, BitPlane::Plane1);
				self.scroll_up(n, BitPlane::Plane2);
			}
		}
	}
	
	// 5YX2: Save V[x] to V[y] in memory starting at I
	fn opcode_5xy2(&mut self, x: u8, y: u8) {
		let first_index = x as usize;
		let last_index = y as usize;
		if first_index <= last_index {
			for i in 0..=(last_index - first_index) {
				let ram_index = (self.i_register + i as u16) as usize;
				self.ram[ram_index] = self.v_register[first_index + i];
			}
		}
		else {
			for i in 0..=(first_index - last_index) {
				let ram_index = (self.i_register + i as u16) as usize;
				self.ram[ram_index] = self.v_register[first_index - i];
			}
		}
		
	}
	// 5YX3: Load V[x] to V[y] from memory starting at I
	fn opcode_5xy3(&mut self, x: u8, y: u8) {
		let first_index = x as usize;
		let last_index = y as usize;
		if first_index <= last_index {
			for i in 0..=(last_index - first_index) {
				let ram_index = (self.i_register + i as u16) as usize;
				self.v_register[first_index + i] = self.ram[ram_index];
			}	
		}
		else {
			for i in 0..(first_index - last_index) {
				let ram_index = (self.i_register + i as u16) as usize;
				self.v_register[first_index - i] = self.ram[ram_index];
			}
		}
		
	}
	// F000: Save the next 16 bits to I
	// NOTE: This command reads 2 opcodes, so we must increment the PC again
	fn opcode_f000(&mut self) {
		let higher_byte = self.ram[self.pc as usize] as u16;
		let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
		let op = (higher_byte << 8) | lower_byte;
		self.i_register = op;
		self.pc += 2;
	}
	// FN01: Select drawing plane(s)
	fn opcode_fn01(&mut self, n: u8) {
		match n {
			0 => self.plane = BitPlane::NoPlane,
			1 => self.plane = BitPlane::Plane1,
			2 => self.plane = BitPlane::Plane2,
			_ => self.plane = BitPlane::Both,
		}
	}
	// F002: Store 16 bytes in audio pattern buffer
	fn opcode_f002(&mut self) {
		for i in 0..16 {
			self.pattern_buffer[i] = self.ram[self.i_register as usize + i];
		}
	}
	
	// FX3A: Set the pitch register to V[x]
	fn opcode_fx3a(&mut self, x: u8) {
		self.pitch = x;
	}
	
}

