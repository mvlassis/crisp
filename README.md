# chip-8-rust
A CHIP-8, SUPER-CHIP, and XO-CHIP emulator written in Rust 

## Features
- All opcodes of the original Chip-8 implemented
- All quirks of the original Chip-8 correctly implemented
- All quirks of the SCHIP-1.1 variant correctly implemented
  - In hires mode, VF is set to the number of rows that include a collision.
- All opcodes of the XO-Chip variant correctly implemented
  - All skip instructions correctly skip the 0xF000 opcode)
- Support for the SCHIP-1.1 variant
- Sound support
- Configuration file to store all settings
- Reset button
- Configurable palettes (and the ability to add your own)

## Controls
The original COSMAC VIP used the 16 hexadecimal digit keys as inputs. The keyboard is mapped to those keys as follows:

`1` `2` `3` `4` -> `1` `2` `3` `C`

`Q` `W` `E` `R` -> `4` `5` `6` `D`

`A` `S` `D` `F` -> `7` `8` `9` `E`

`Z` `X` `C` `V` -> `A` `0` `B` `F`

| Key | Action |
| ---| --- |
|`UP` |Increase tick rate by 5|
|`DOWN` |Decrease tick rate by 5|
|`RIGHT`| Pick next color theme |
|`LEFT`| Pick previous color theme |
|`M`| Mute/Unmute|
|`O`| Save state|
|`I`| Load last save state|
|`P`| Reset Emulator
|`ESC` | Exit |


## Options
`-m` Start the program muted

## TODO List
- ~~Add sound support~~
- ~~Add configration file to store settings~~
- Add customization options
- Add pause/resume button
- ~~Add reset button~~
- ~~Support SCHIP-1.1 variant~~
- Support XO-CHIP variant

## Acknowledgements

## License
