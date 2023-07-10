use clap::Parser;
use clap::ValueEnum;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum CLIVariant {
	Chip8,
	SChip,
	XOChip,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {

	// The file name we read, must be given as an argument
	pub file_name: String,

	// Emulation Settings
	#[arg(short, long, value_enum, default_value_t = CLIVariant::SChip)]
	pub variant: CLIVariant,

	#[arg(long)]
	pub quirk_vfreset: bool,
	#[arg(long)]
	pub quirk_memory: bool,
	#[arg(long)]
	pub quirk_displaywait: bool,
	#[arg(long)]
	pub quirk_clipping: bool,
	#[arg(long)]
	pub quirk_shifting: bool,
	#[arg(long)]
	pub quirk_jumping: bool,
	#[arg(long)]
	pub quirk_clipcollision: bool,
	
	// Display settings
	// How many cycles are executed per frame
	// Note: This is not actually related to emulation
	#[arg(short, long, default_value_t = 15)]
	pub ticks_per_frame: u32,

	// The multiplier by which we scale the display
	#[arg(short, long, default_value_t = 10)]
	pub scale: u8,

	// Whether we want to turn fps capping off
	#[arg(long)]
	pub toggle_fpscap: bool,

	// Audio settings
	#[arg(short, long)]
	pub mute: bool, // Start muted
}

impl Args {
	// Returns the variant given from the CLI
	pub fn get_variant(&self) -> chip8_core::Variant {
		match self.variant {
			CLIVariant::Chip8 => chip8_core::Variant::Chip8,
			CLIVariant::SChip => chip8_core::Variant::SChip,
			CLIVariant::XOChip => chip8_core::Variant::XOChip,
		}
	}

	pub fn get_fpscap(&self) -> bool {
		let mut fpscap = match self.variant {
			CLIVariant::Chip8 => false,
			CLIVariant::SChip => true,
			CLIVariant::XOChip => true,
		};
		if self.toggle_fpscap {
			fpscap = !fpscap;
		}
		fpscap
	}
	
	// Returns an emulation config from the arguments given
	pub fn get_emuconfig(&self) -> chip8_core::EmuConfig {
		let mut emu_config = match self.variant {
			CLIVariant::Chip8 => {
				chip8_core::EmuConfig {
					variant: chip8_core::Variant::Chip8,
					quirk_vfreset: true,
					quirk_memory: true,
					quirk_displaywait: true,
					quirk_clipping: true,
					quirk_shifting: false,
					quirk_jumping: false,
					quirk_clipcollision: false,
				}
			}
			CLIVariant::SChip => {
				chip8_core::EmuConfig {
					variant: chip8_core::Variant::SChip,
					quirk_vfreset: false,
					quirk_memory: false,
					quirk_displaywait: true,
					quirk_clipping: true,
					quirk_shifting: true,
					quirk_jumping: true,
					quirk_clipcollision: false,
				}
			}
			CLIVariant::XOChip => {
				chip8_core::EmuConfig {
					variant: chip8_core::Variant::XOChip,
					quirk_vfreset: false,
					quirk_memory: true,
					quirk_displaywait: false,
					quirk_clipping: false,
					quirk_shifting: false,
					quirk_jumping: false,
					quirk_clipcollision: false,
				}
			}
		};
		if self.quirk_vfreset  {
			emu_config.quirk_vfreset = !emu_config.quirk_vfreset;
		}
		if self.quirk_memory  {
			emu_config.quirk_memory = !emu_config.quirk_memory;
		}
		if self.quirk_displaywait  {
			emu_config.quirk_displaywait = !emu_config.quirk_displaywait;
		}
		if self.quirk_clipping  {
			emu_config.quirk_clipping = !emu_config.quirk_clipping;
		}
		if self.quirk_shifting  {
			emu_config.quirk_shifting = !emu_config.quirk_shifting;
		}
		if self.quirk_jumping  {
			emu_config.quirk_jumping = !emu_config.quirk_jumping;
		}
		if self.quirk_clipcollision  {
			emu_config.quirk_clipcollision = !emu_config.quirk_clipcollision;
		}
		emu_config
	}
}
