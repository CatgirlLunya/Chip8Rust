extern crate sdl2;

use std::collections::HashMap;
use sdl2::keyboard::Scancode;
use sdl2::render::TextureAccess;
use sdl2::video::WindowContext;
use rand::Rng;

const FONT: [u8; 80] = [
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

pub struct Chip {
    pub speed: u32, // Ops per second
    pub input: [bool; 16],
    pub seeking_input: bool,
    pub key_pressed: Scancode,

    ram: [u8; 4096],
    display: [u8; 4 * 64 * 32],
    index: u16,
    pc: u16,
    stack: Vec<u16>,
    delay_timer: u8,
    audio_timer: u8,
    registers: [u8; 16],
    texture_creator: sdl2::render::TextureCreator<WindowContext>,
    needs_to_update: bool,
}

impl Chip {
    pub fn new(canvas: &sdl2::render::WindowCanvas) -> Self {
        let mut chip = Chip{
            speed: 700,
            input: [false; 16],
            seeking_input: false,
            key_pressed: Scancode::Cancel, // Dummy scancode

            ram: [0; 4096],
            display: [0; 4 * 64 * 32],
            index: 0,
            pc: 0x200,
            stack: vec![],
            delay_timer: 0,
            audio_timer: 0,
            registers: [0; 16],

            texture_creator: canvas.texture_creator(),
            needs_to_update: false,
        };
        for i in 0..80 {
            chip.ram[i] = FONT[i];
        };
        return chip;
    }

    pub fn get_input_map() -> HashMap<Scancode, u8> {
        HashMap::from([
            (Scancode::Kp0, 0),
            (Scancode::Kp1, 1),
            (Scancode::Kp2, 2),
            (Scancode::Kp3, 3),
            (Scancode::Kp4, 4),
            (Scancode::Kp5, 5),
            (Scancode::Kp6, 6),
            (Scancode::Kp7, 7),
            (Scancode::Kp8, 8),
            (Scancode::Kp9, 9),
            (Scancode::KpA, 10),
            (Scancode::KpB, 11),
            (Scancode::KpC, 12),
            (Scancode::KpD, 13),
            (Scancode::KpE, 14),
            (Scancode::KpE, 15),
        ])
    }

    pub fn load_rom(&mut self, bytes: Vec<u8>) {
        let len = bytes.len();
        for i in 0..len {
            self.ram[i + 0x200] = bytes[i];
        }
    }

    fn clear_screen(&mut self) {
        for y in 0..32 {
            for x in 0..64 {
                self.set_pixel(x, y, false);
            }
        }
    }

    fn set_pixel(&mut self, x: usize, y: usize, color: bool) {
        if x * 4 + y * 64 * 4 >= 8192 {
            return;
        }
        for i in 0..4 {
            // 4 bytes per pixel, 64 * 4 bytes per row
            self.display[i + x * 4 + y * 64 * 4] = color as u8 * 255;
        }
    }

    fn get_pixel(&mut self, x: usize, y: usize) -> bool {
        if x * 4 + y * 64 * 4 >= 8192 {
            return false;
        }
        self.display[x * 4 + y * 64 * 4] != 0
    }

    fn fetch(&mut self) -> u16 {
        let instruction = ((self.ram[self.pc as usize] as u16) << 8) + (self.ram[(self.pc + 1) as usize] as u16);
        self.pc += 2;
        return instruction;
    }

    //noinspection ALL
    fn decode(&mut self, opcode: u16) {
        let x_reg = (opcode >> 8 & 0xF) as usize;
        let y_reg = (opcode >> 4 & 0xF) as usize;
        let n = (opcode & 0xF) as u8;
        let nn = (opcode & 0xFF) as u8;
        let nnn = (opcode & 0xFFF) as u16;

        match opcode >> 12 & 0xF {
            0 => {
                if nnn == 0x0E0 {
                    println!("Clear Screen!");
                    self.needs_to_update = true;
                    self.clear_screen();
                }
                if nnn == 0x0EE {
                    self.pc = self.stack.pop().unwrap();
                }
            } // RIGHT
            1 => {
                self.pc = nnn;
            } // RIGHT
            2 => {
                self.stack.push(self.pc);
                self.pc = nnn;
            } // RIGHT
            3 => {
                if self.registers[x_reg] == nn {
                    self.pc += 2;
                }
            } // RIGHT
            4 => {
                if self.registers[x_reg] != nn {
                    self.pc += 2;
                }
            } // RIGHT
            5 => {
                if self.registers[x_reg] == self.registers[y_reg] {
                    self.pc += 2;
                }
            } // RIGHT
            6 => {
                self.registers[x_reg] = nn;
            } // RIGHT
            7 => {
                self.registers[x_reg] = self.registers[x_reg].wrapping_add(nn);
            } // RIGHT
            8 => {
                match n {
                    0 => {
                        self.registers[x_reg] = self.registers[y_reg];
                    }
                    1 => {
                        self.registers[x_reg] |= self.registers[y_reg];
                    }
                    2 => {
                        self.registers[x_reg] &= self.registers[y_reg];
                    }
                    3 => {
                        self.registers[x_reg] ^= self.registers[y_reg];
                    }
                    4 => {
                        let (result, overflow) = self.registers[x_reg].overflowing_add(self.registers[y_reg]);
                        self.registers[x_reg] = result;
                        self.registers[0xF] = overflow as u8;
                    }
                    5 => {
                        let (result, overflow) = self.registers[x_reg].overflowing_sub(self.registers[y_reg]);
                        self.registers[x_reg] = result;
                        self.registers[0xF] = !overflow as u8;
                    }
                    6 => {
                        self.registers[0xF] = self.registers[x_reg] & 1;
                        self.registers[x_reg] >>= 1;
                    }
                    7 => {
                        let (result, overflow) = self.registers[y_reg].overflowing_sub(self.registers[x_reg]);
                        self.registers[x_reg] = result;
                        self.registers[0xF] = !overflow as u8;
                    }
                    0xE => {
                        self.registers[0xF] = self.registers[x_reg] >> 7;
                        self.registers[x_reg] <<= 1;
                    }
                    _ => {}
                }
            }
            9 => {
                if self.registers[x_reg] != self.registers[y_reg] {
                    self.pc += 2;
                }
            } // RIGHT
            0xA => {
                self.index = nnn;
            } // RIGHT
            0xB => {
                self.pc = nnn + self.registers[0] as u16;
            } // RIGHT
            0xC => {
                self.registers[x_reg] = nn & rand::thread_rng().gen::<u8>();
            } // RIGHT
            0xD => {
                self.needs_to_update = true;
                let x_coord = (self.registers[x_reg] & 63) as usize;
                let y_coord = (self.registers[y_reg] & 31) as usize;
                self.registers[0xF] = 0;

                for row in 0..n {
                    let sprite_byte = self.ram[(self.index + row as u16) as usize];
                    for bit in 0..8 {
                        // For some reason, bits go 76543210 order
                        let bit_value = (sprite_byte >> (7 - bit) & 1) != 0;
                        if bit_value {
                            if self.get_pixel(x_coord + bit, y_coord + row as usize) {
                                self.registers[0xF] = 1;
                                self.set_pixel(x_coord + bit, y_coord + row as usize, false);
                            } else {
                                self.set_pixel(x_coord + bit, y_coord + row as usize, true);
                            }
                        }
                        if x_coord + bit >= 64 {
                            break;
                        }
                    }
                    if y_coord + row as usize >= 32 {
                        break;
                    }
                }
            } // RIGHT
            0xE => {
                match nn {
                    0x9E => {
                        if self.input[self.registers[x_reg] as usize] {
                            self.pc += 2;
                        }
                    }
                    0xA1 => {
                        if !self.input[self.registers[x_reg] as usize] {
                            self.pc += 2;
                        }
                    }
                    _ => {}
                }
            } // RIGHT
            0xF => {
                match nn {
                    0x07 => {
                        self.registers[x_reg] = self.delay_timer;
                    } // RIGHT
                    0x0A => {
                        if self.key_pressed == Scancode::Cancel {
                            self.seeking_input = true;
                            self.pc -= 2;
                        } else {
                            self.registers[x_reg] = Chip::get_input_map()[&self.key_pressed];
                            self.seeking_input = false;
                            self.key_pressed = Scancode::Cancel;
                        }
                    } // RIGHT
                    0x15 => {
                        self.delay_timer = self.registers[x_reg];
                    } // RIGHT
                    0x18 => {
                        self.audio_timer = self.registers[x_reg];
                    } // RIGHT
                    0x1E => {
                        self.index += self.registers[x_reg] as u16;
                        if self.index >= 0x1000 {
                            self.registers[0xF] = 1;
                        }
                    } // RIGHT
                    0x29 => {
                        self.index = ((self.registers[x_reg] & 0xF) * 5) as u16;
                    } // RIGHT
                    0x33 => {
                        let value = self.registers[x_reg];
                        self.ram[self.index as usize] = value / 100;
                        self.ram[self.index as usize + 1] = (value / 10) % 10;
                        self.ram[self.index as usize + 2] = value % 100;
                    } // RIGHT
                    // TODO: FIX TO BE CONFIGURABLE
                    0x55 => {
                        for reg in 0..=x_reg {
                            self.ram[self.index as usize] = self.registers[reg];
                            self.index += 1;
                        }
                    }
                    0x65 => {
                        for reg in 0..=x_reg {
                            self.registers[reg] = self.ram[self.index as usize];
                            self.index += 1;
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        };
    }

    pub fn render(&mut self, canvas: &mut sdl2::render::WindowCanvas) {
        if !self.needs_to_update {
            return;
        }
        let mut texture = self.texture_creator
            .create_texture(None, TextureAccess::Static, 64, 32)
            .unwrap();
        texture.update(None, &self.display, 64 * 4).expect("Failed to update texture!");
        canvas.copy(&texture, None, None).expect("Failed to copy Texture to Canvas");
        canvas.present();
    }

    pub fn update(&mut self) {
        let opcode = self.fetch();
        self.decode(opcode);
    }

    pub fn dec_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.audio_timer > 0 {
            self.audio_timer -= 1;
        }
    }
}
