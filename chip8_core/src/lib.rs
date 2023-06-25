use rand::Rng;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

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
0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Emulator {
	pc: u16,
	ram: [u8; RAM_SIZE],
	screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
	// FYI: v stands for "variable"
	v_register: [u8; NUM_REGISTERS],
	// The I register only needs 12 bits
	i_register: u16,
	stack_pointer: i16,
	stack: [u16; STACK_SIZE],
	keys: [bool; NUM_KEYS],
	delay_timer: u8,
	sound_timer: u8,
	pub beep: bool,
}

const START_ADDRESS: u16 = 0x200;

impl Emulator {
	pub fn new() -> Self {
		let mut new_emulator = Self {
			pc: START_ADDRESS,
			ram: [0; RAM_SIZE],
			screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
			v_register: [0; NUM_REGISTERS],
			i_register: 0,
			stack_pointer: -1,
			stack: [0; STACK_SIZE],
			keys: [false; NUM_KEYS],
			delay_timer: 0,
			sound_timer: 0,
			beep: false,
		};

		new_emulator.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
		new_emulator
	}
	
	fn push(&mut self, value: u16) {
		self.stack_pointer += 1;
		self.stack[self.stack_pointer as usize] = value;
	}		
	
	fn pop(&mut self) -> u16 {
		self.stack_pointer -= 1;
		self.stack[(self.stack_pointer + 1) as usize]
	}

	pub fn reset(&mut self) {
		self.pc = START_ADDRESS;
		self.ram = [0; RAM_SIZE];
		self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
		self.v_register = [0; NUM_REGISTERS];
		self.i_register = 0;
		self.stack_pointer = -1;
		self.stack = [0; STACK_SIZE];
		self.keys = [false; NUM_KEYS];
		self.delay_timer = 0;
		self.sound_timer = 0;
		self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
	}
	
	pub fn tick(&mut self) {
		// Fetch
		let op = self.fetch();
		// Decode and Execute
		self.execute(op);
	}

	pub fn get_display(&self) -> &[bool]{
		&self.screen
	}

	pub fn register_keypress(&mut self, index: usize, pressed: bool) {
		self.keys[index] = pressed;
	}

	pub fn load(&mut self, data: &[u8]) {
		let start = START_ADDRESS as usize;
		let end = start + data.len();
		self.ram[start..end].copy_from_slice(data);
	}
	
	fn fetch(&mut self) -> u16 {
		let higher_byte = self.ram[self.pc as usize] as u16;
		let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
		let op = (higher_byte << 8) | lower_byte;
		self.pc += 2;
		op
	}

	fn execute(&mut self, op:u16) {
		let digit1 = (op & 0xF000) >> 12;
		let digit2 = (op & 0x0F00) >> 8;
		let digit3 = (op & 0x00F0) >> 4;
		let digit4 = op & 0x000F;

		match (digit1, digit2, digit3, digit4) {
			// 0000: Nop
			(0x0, 0x0, 0x0, 0x0) => {
				return;
			}
			// 00E0: Clear screen
			(0x0, 0x0, 0xE, 0x0) => {
				self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
			},
			// 00EE: Return from subroutine
			(0x0, 0x0, 0xE, 0xE) => {
				let ret_addr = self.pop();
				self.pc = ret_addr;
			}
			// 1MMM: Jump
			(0x1, _, _, _) =>  {
				let next_instruction = op & 0xFFF;
				self.pc = next_instruction;
			}
			// 2MMM: Call subroutine
			(0x2, _, _, _) => {
				let next_instruction = op & 0xFFF;
				self.push(self.pc);
				self.pc = next_instruction;				
			}
			// 3XNN: Skip if V[x] == NN
			(0x3, _, _, _) => {
				let v_number = digit2 as usize;
				let number_to_compare = (op & 0xFF) as u8;
				if self.v_register[v_number] == number_to_compare {
					self.pc += 2;
				}
			}
			// 4XNN: Skip if V[x] != NN
			(0x4, _, _, _) => {
				let v_index = digit2 as usize;
				let number_to_compare = (op & 0xFF) as u8;
				if self.v_register[v_index] != number_to_compare {
					self.pc += 2;
				}
			}
			// 5XY0: Skip if V[x] == V[y]
			(0x5, _, _, 0) => {
				let v_index1 = digit2 as usize;
				let v_index2 = digit3 as usize;
				if self.v_register[v_index1] == self.v_register[v_index2] {
					self.pc += 2;
				}
			}
			// 6XNN: V[x] = NN
			(0x6, _, _, _) => {
				let v_index = digit2 as usize;
				let value = (op & 0xFF) as u8;
				self.v_register[v_index] = value;
			}
			// 7XNN: V[x] = V[x] + NN
			(0x7, _, _, _) => {
				let v_index = digit2 as usize;
				let value = (op & 0xFF) as u8;
				self.v_register[v_index] = self.v_register[v_index].wrapping_add(value);
			}
			// 8XY0: V[x] = V[y]
			(0x8, _, _, 0) => {
				let v_index1 = digit2 as usize;
				let v_index2 = digit3 as usize;
				self.v_register[v_index1] = self.v_register[v_index2];
			}
			// 8YX1; V[x] = V[x] OR V[y]
			(0x8, _, _, 1) => {
				let v_index1 = digit2 as usize;
				let v_index2 = digit3 as usize;
				self.v_register[v_index1] = self.v_register[v_index1] | self.v_register[v_index2];
			}
			// 8XY2: V[x] = V[x] AND V[y]
			(0x8, _, _, 2) => {
				let v_index1 = digit2 as usize;
				let v_index2 = digit3 as usize;
				self.v_register[v_index1] = self.v_register[v_index1] & self.v_register[v_index2];
			}
			// 8XY3: V[x] = V[x] XOR V[y]
			(0x8, _, _, 3) => {
				let v_index1 = digit2 as usize;
				let v_index2 = digit3 as usize;
				self.v_register[v_index1] = self.v_register[v_index1] ^ self.v_register[v_index2];
			}
			// 8XY4: V[x] = V[x] + V[y], V[F] = 1 if overflow
			(0x8, _, _, 4) => {
				let v_index1 = digit2 as usize;
				let v_index2 = digit3 as usize;
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
			(0x8, _, _, 5) => {
				let v_index1 = digit2 as usize;
				let v_index2 = digit3 as usize;
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
			(0x8, _, _, 6) => {
				let index1 = digit2 as usize;
				let index2 = digit3 as usize;
				// TODO: QUIRK OPTION
				self.v_register[index1] = self.v_register[index2];
				let lsb = self.v_register[index1] & 1;
				self.v_register[index1] >>= 1;
				self.v_register[0xF] = lsb;
			}
			// 8XY7: V[x] = V[y] - V[x]
			(0x8, _, _, 7) => {
				let v_index1 = digit2 as usize;
				let v_index2 = digit3 as usize;
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
			(0x8, _, _, 0xE) => {
				let index1 = digit2 as usize;
				let index2 = digit3 as usize;
				self.v_register[index1] = self.v_register[index2];
				let msb = (self.v_register[index1] >> 7) & 1;
				self.v_register[index1] <<= 1;
				self.v_register[0xF] = msb;
			}			
			// 9XY0: Skip if V[x] != V[y]
			(0x9, _, _, 0) => {
				let v_index1 = digit2 as usize;
				let v_index2 = digit3 as usize;
				if self.v_register[v_index1] != self.v_register[v_index2] {
					self.pc += 2;
				}
			}
			// AMMM: I = MMM
			(0xA, _, _, _) => {
				let index = op & 0xFFF;
				self.i_register = index;
			}
			// BMMM: Jump to MMM + V[0]
			(0xB, _, _, _) => {
				let index = (op & 0xFFF) as u16;
				self.pc = index + (self.v_register[0] as u16);
			}
			// CXNN: Get random byte, then AND with NN
			(0xC, _, _, _) => {
				let random_byte = rand::thread_rng().gen_range(0..=255);
				let index = digit2 as usize;
				let value = (op & 0xFF) as u8;
				self.v_register[index] = value & random_byte;
			}
			// DXYN: Display sprite of N rows at coordinates V[x], V[y]
			(0xD, _, _, _) => {
                let mut flipped = false;
                // Get the base (x, y) coords
                let x_base = self.v_register[digit2 as usize] as u16;
                let y_base = self.v_register[digit3 as usize] as u16;
                // The last digit determines how many rows high our sprite is
                let num_rows = digit4;
                for row in 0..num_rows {
                    // Determine which memory address our row's data is stored
                    let address = self.i_register + row as u16;
                    let pixels = self.ram[address as usize];
                    for column in 0..8 {
                        // Use a mask to fetch current pixel's bit. Only flip if a 1
                        if (pixels & (0b1000_0000 >> column)) != 0 {
                            // Sprites should wrap around screen, so apply modulo
                            let x = (x_base + column) as usize % SCREEN_WIDTH;
                            let y = (y_base + row) as usize % SCREEN_HEIGHT;

                            let index = x + SCREEN_WIDTH * y;
                            flipped |= self.screen[index];
                            self.screen[index] ^= true;
                        }
                    }
                }
                if flipped {
                    self.v_register[0xF] = 1;
                } else {
                    self.v_register[0xF] = 0;
                }
            }
			// EX9E - Skip if key VX is pressed
			(0xE, _, 0x9, 0xE) => {
				let index = digit2 as usize;
				let vx = self.v_register[index];
				let key = self.keys[vx as usize];
				if key {
					self.pc += 2;
				}
			}
			// EXA1 - Skip if key VX is not pressed
			(0xE, _, 0xA, 0x1) => {
				let index = digit2 as usize;
				let vx = self.v_register[index];
				let key = self.keys[vx as usize];
				if !key {
					self.pc += 2;
				}
			}
			// FX07 - VX = Time: Get current timer value
			(0xF, _, 0x0, 0x7) => {
				let index = digit2 as usize;
				self.v_register[index] = self.delay_timer;
			}
			// FX0A - Wait for key press
			(0xF, _, 0x0, 0xA) => {
				let mut pressed = false;
				let index = digit2 as usize;
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
			(0xF, _, 0x1, 0x5) => {
				let index = digit2 as usize;
				self.delay_timer = self.v_register[index];
			}
			// FX18: Initialize sound timer
			(0xF, _, 0x1, 0x8) => {
				let index = digit2 as usize;
				self.sound_timer = self.v_register[index];
			}
			// FX1E: Add V[x] to the memory pointer I
			(0xF, _, 0x1, 0xE) => {
				let index = digit2 as usize;
				self.i_register = self.i_register.wrapping_add(self.v_register[index] as u16);
			}
			// FX29: Set I to show digit V[x]
			(0xF, _, 0x2, 0x9) => {
				let index = digit2 as usize;
				let character = self.v_register[index] as u16;
				self.i_register = character * 5;
					
			}
			// FX33: Store 3 digits of V[x] at M[I]
			// TODO: Optimize BCD
			(0xF, _, 3, 3) => {
				let index = digit2 as usize;
				let value = self.v_register[index];
				let ones = value % 10;
				let tens = (value / 10) % 10;
				let hundreds = value / 100;
				self.ram[self.i_register as usize] = hundreds;
				self.ram[(self.i_register+1) as usize] = tens;
				self.ram[(self.i_register+2) as usize] = ones;
			}
			// FX55: Store V[0] to V[x] at M[I]
			(0xF, _, 5, 5) => {
				let last_index = digit2 as usize;
				for i in 0..=last_index {
					let ram_index = (self.i_register + i as u16) as usize;
					self.ram[ram_index] = self.v_register[i];
				}
				self.i_register = self.i_register + last_index as u16 + 1;
			}
			// FX65: Load V[0] to V[x] from M[I]
			(0xF, _, 6, 5) => {
				let last_index = digit2 as usize;
				for i in 0..=last_index {
					let ram_index = (self.i_register + i as u16) as usize;
					self.v_register[i] = self.ram[ram_index];
				}
				self.i_register = self.i_register + last_index as u16 + 1;				
			}						
			(_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
		}
	}

	pub fn tick_timers(&mut self) {
		if self.delay_timer > 0 {
			self.delay_timer -= 1;
		}

		if self.sound_timer > 0 {
			self.sound_timer -= 1;
			self.beep = true;

			if self.sound_timer == 0 {
				self.beep = false;
			}
		}
	}


	// Helper function to print the screen state
	pub fn print_screen(&self) {
		for i in 0..self.screen.len() {
			let a = if self.screen[i] == true {"X"} else {" "};
			print!("{}", a);
			if i % (SCREEN_WIDTH) == 0 {
				println!("");
			}
		}
		println!("");
 		println!("****************************************************************");
	}
}


