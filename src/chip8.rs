struct Opcode {
    code: u16,
    n: u8,
    nn: u8,
    nnn: u16,
    x: u8,
    y: u8,
}
impl Opcode {
    fn new(code: u16) -> Opcode {
        Opcode {
            code: code,
            n: (code & 0xf) as u8,
            nn: (code & 0xff) as u8,
            nnn: code & 0xfff,
            x: ((code & 0x0f00) >> 8) as u8,
            y: ((code & 0x00f0) >> 4) as u8,
        }
    }
}

pub struct Chip8 {
    registers: [u8; 16], // V
    index: usize,        // I
    pub delay: u8,
    pub sound: u8,
    program_counter: usize, // PC
    stack_pointer: usize,   // SP
    stack: [usize; 16],
    pub memory: [u8; 0xfff],
    pub screen_size: (u8, u8),
    pub screen: Vec<u8>,
    pub other_mode: bool,
    pub keys: [bool; 16],
    pub has_drawn: bool,
    pub has_handled_draw: bool,
}

impl Chip8 {
    #[rustfmt::skip]
    pub fn new() -> Chip8 {
        let mut chip8 = Chip8 {
            registers: [0; 16],
            index: 0,
            delay: 0,
            sound: 0,
            program_counter: 0x200,
            stack_pointer: 0,
            stack: [0; 16],
            memory: [0; 0xfff],
            screen_size: (64, 32),
            screen: Vec::new(),
            other_mode: false,
            keys: [false; 16],
            has_drawn: false,
            has_handled_draw: false,
        };
        chip8.screen.resize((64 / 8) * 32, 0);

        let mut letter: [u8;5];

        // 0
        letter = [
            0b11110000,
            0b10010000,
            0b10010000,
            0b10010000,
            0b11110000
            ];
        chip8.add_letter(0x0, &letter);

        // 1
        letter = [
            0b00100000,
            0b01100000,
            0b00100000,
            0b00100000,
            0b01110000
            ];
        chip8.add_letter(0x1, &letter);

        // 2
        letter = [
            0b11110000,
            0b00010000,
            0b11110000,
            0b10000000,
            0b11110000
            ];
        chip8.add_letter(0x2, &letter);

        // 3
        letter = [
            0b11110000,
            0b00010000,
            0b11110000,
            0b00010000,
            0b11110000
            ];
        chip8.add_letter(0x3, &letter);

        // 4
        letter = [
            0b10010000,
            0b10010000,
            0b11110000,
            0b00010000,
            0b00010000
            ];
        chip8.add_letter(0x4, &letter);

        // 5
        letter = [
            0b11110000,
            0b10000000,
            0b11110000,
            0b00010000,
            0b11110000
            ];
        chip8.add_letter(0x5, &letter);

        // 6
        letter = [
            0b11110000,
            0b10000000,
            0b11110000,
            0b10010000,
            0b11110000
            ];
        chip8.add_letter(0x6, &letter);

        // 7
        letter = [
            0b11110000,
            0b00010000,
            0b00100000,
            0b01000000,
            0b01000000
            ];
        chip8.add_letter(0x7, &letter);

        // 8
        letter = [
            0b11110000,
            0b10010000,
            0b11110000,
            0b10010000,
            0b11110000
            ];
        chip8.add_letter(0x8, &letter);

        // 9
        letter = [
            0b11110000,
            0b10010000,
            0b11110000,
            0b00010000,
            0b11110000
            ];
        chip8.add_letter(0x9, &letter);

        // a
        letter = [
            0b11110000,
            0b10010000,
            0b11110000,
            0b10010000,
            0b10010000
            ];
        chip8.add_letter(0xa, &letter);

        // b
        letter = [
            0b11100000,
            0b10010000,
            0b11100000,
            0b10010000,
            0b11100000
            ];
        chip8.add_letter(0xb, &letter);

        // c
        letter = [
            0b11110000,
            0b10000000,
            0b10000000,
            0b10000000,
            0b11110000
            ];
        chip8.add_letter(0xc, &letter);

        // d
        letter = [
            0b11100000,
            0b10010000,
            0b10010000,
            0b10010000,
            0b11100000
            ];
        chip8.add_letter(0xd, &letter);

        // e
        letter = [
            0b11110000,
            0b10000000,
            0b11110000,
            0b10000000,
            0b11110000
            ];
        chip8.add_letter(0xe, &letter);

        // f
        letter = [
            0b11110000,
            0b10000000,
            0b11110000,
            0b10000000,
            0b10000000
            ];
        chip8.add_letter(0xf, &letter);

        chip8
    }

    fn add_letter(&mut self, letter: usize, sprite: &[u8; 5]) {
        let offset: usize = letter * 5;
        for i in 0 as usize..5 {
            self.memory[offset + i] = sprite[i];
        }
    }

    pub fn clock(&mut self) {
        let code: u16 = (self.memory[self.program_counter] as u16) << 8
            | self.memory[self.program_counter + 1] as u16;
        let opcode = Opcode::new(code);
        // println!("{:#06x}: {:#06x}", self.program_counter, code);

        if self.has_handled_draw {
            self.has_drawn = false;
        }

        match opcode.code {
            0x00e0 => self.cls(opcode),
            0x00ee => self.ret(opcode),
            _ => match opcode.code >> 12 {
                0x1 => self.jp(opcode),
                0x2 => self.call(opcode),
                0x3 => self.se(opcode),
                0x4 => self.sne(opcode),
                0x5 => match opcode.code & 0xf {
                    0x0 => self.sey(opcode),
                    _ => panic!("Unknown opcode: {:#06x}", opcode.code),
                },
                0x6 => self.ld(opcode),
                0x7 => self.add(opcode),
                0x8 => match opcode.code & 0xf {
                    0x0 => self.ldy(opcode),
                    0x1 => self.or(opcode),
                    0x2 => self.and(opcode),
                    0x3 => self.xor(opcode),
                    0x4 => self.addy(opcode),
                    0x5 => self.sub(opcode),
                    0x6 => {
                        if self.other_mode {
                            self.shr(opcode);
                        } else {
                            self.shry(opcode);
                        }
                    }
                    0x7 => self.subn(opcode),
                    0xe => {
                        if self.other_mode {
                            self.shl(opcode);
                        } else {
                            self.shly(opcode);
                        }
                    }
                    _ => panic!("Unknown opcode: {:#06x}", opcode.code),
                },
                0x9 => match opcode.code & 0xf {
                    0x0 => self.sney(opcode),
                    _ => panic!("Unknown opcode: {:#06x}", opcode.code),
                },
                0xa => self.ldi(opcode),
                0xb => self.jp0(opcode),
                0xc => self.rnd(opcode),
                0xd => self.drw(opcode),
                0xe => match opcode.code & 0xff {
                    0x9e => self.skp(opcode),
                    0xa1 => self.skpn(opcode),
                    _ => panic!("Unknown opcode: {:#06x}", opcode.code),
                },
                0xf => match opcode.code & 0xff {
                    0x07 => self.ldxdt(opcode),
                    0x0a => self.ldk(opcode),
                    0x15 => self.lddt(opcode),
                    0x18 => self.ldst(opcode),
                    0x1e => self.addi(opcode),
                    0x29 => self.ldf(opcode),
                    0x33 => self.ldb(opcode),
                    0x55 => self.ldix(opcode),
                    0x65 => self.ldxi(opcode),
                    _ => panic!("Unknown opcode: {:#06x}", opcode.code),
                },
                _ => panic!("Unknown opcode: {:#06x}", opcode.code),
            },
        }

        self.program_counter += 2;
    }

    // 00e0
    fn cls(&mut self, _opcode: Opcode) {
        self.has_drawn = true;
        self.has_handled_draw = false;
        for pixel in self.screen.iter_mut() {
            *pixel = 0;
        }
    }

    // 00ee
    fn ret(&mut self, _opcode: Opcode) {
        self.program_counter = self.stack[self.stack_pointer];
        self.stack_pointer -= 1;
    }

    // 1nnn
    fn jp(&mut self, opcode: Opcode) {
        self.program_counter = opcode.nnn as usize - 2;
    }

    // 2nnn
    fn call(&mut self, opcode: Opcode) {
        self.stack_pointer += 1;
        self.stack[self.stack_pointer] = self.program_counter;
        self.program_counter = opcode.nnn as usize - 2;
    }

    // 3xnn
    fn se(&mut self, opcode: Opcode) {
        if self.registers[opcode.x as usize] == opcode.nn {
            self.program_counter += 2;
        }
    }

    // 4xnn
    fn sne(&mut self, opcode: Opcode) {
        if self.registers[opcode.x as usize] != opcode.nn {
            self.program_counter += 2;
        }
    }

    // 5xy0
    fn sey(&mut self, opcode: Opcode) {
        if self.registers[opcode.x as usize] == self.registers[opcode.y as usize] {
            self.program_counter += 2;
        }
    }

    // 6xnn
    fn ld(&mut self, opcode: Opcode) {
        self.registers[opcode.x as usize] = opcode.nn;
    }

    // 7xnn
    fn add(&mut self, opcode: Opcode) {
        let x = &mut self.registers[opcode.x as usize];
        *x = x.wrapping_add(opcode.nn);
    }

    // 8xy0
    fn ldy(&mut self, opcode: Opcode) {
        self.registers[opcode.x as usize] = self.registers[opcode.y as usize];
    }

    // 8xy1
    fn or(&mut self, opcode: Opcode) {
        self.registers[opcode.x as usize] |= self.registers[opcode.y as usize];
    }

    // 8xy2
    fn and(&mut self, opcode: Opcode) {
        self.registers[opcode.x as usize] &= self.registers[opcode.y as usize];
    }

    // 8xy3
    fn xor(&mut self, opcode: Opcode) {
        self.registers[opcode.x as usize] ^= self.registers[opcode.y as usize];
    }

    // 8xy4
    fn addy(&mut self, opcode: Opcode) {
        self.registers[0xf] = 0;
        let result =
            self.registers[opcode.x as usize].overflowing_add(self.registers[opcode.y as usize]);
        self.registers[opcode.x as usize] = result.0;
        if result.1 {
            self.registers[0xf] = 1;
        }
    }

    // 8xy5
    fn sub(&mut self, opcode: Opcode) {
        self.registers[0xf] = 0;
        let result =
            self.registers[opcode.x as usize].overflowing_sub(self.registers[opcode.y as usize]);
        self.registers[opcode.x as usize] = result.0;
        if result.1 {
            self.registers[0xf] = 1;
        }
    }

    // 8x06
    fn shr(&mut self, opcode: Opcode) {
        self.registers[0xf] = 0;
        if self.registers[opcode.x as usize] & 0b1 == 1 {
            self.registers[0xf] = 1;
        }
        self.registers[opcode.x as usize] = self.registers[opcode.x as usize] >> 1;
    }

    // 8xy6
    fn shry(&mut self, opcode: Opcode) {
        self.registers[0xf] = 0;
        if self.registers[opcode.y as usize] & 0b1 == 1 {
            self.registers[0xf] = 1;
        }
        self.registers[opcode.x as usize] = self.registers[opcode.y as usize] >> 1;
    }

    // 8xy7
    fn subn(&mut self, opcode: Opcode) {
        self.registers[0xf] = 0;
        let result =
            self.registers[opcode.y as usize].overflowing_sub(self.registers[opcode.x as usize]);
        self.registers[opcode.x as usize] = result.0;
        if result.1 {
            self.registers[0xf] = 1;
        }
    }

    // 8x0e
    fn shl(&mut self, opcode: Opcode) {
        self.registers[0xf] = 0;
        if self.registers[opcode.x as usize] & 0b10000000 != 0 {
            self.registers[0xf] = 1;
        }
        self.registers[opcode.x as usize] = self.registers[opcode.x as usize] << 1;
    }

    // 8xye
    fn shly(&mut self, opcode: Opcode) {
        self.registers[0xf] = 0;
        if self.registers[opcode.y as usize] & 0b10000000 != 0 {
            self.registers[0xf] = 1;
        }
        self.registers[opcode.x as usize] = self.registers[opcode.y as usize] << 1;
    }

    // 9xy0
    fn sney(&mut self, opcode: Opcode) {
        if self.registers[opcode.x as usize] != self.registers[opcode.y as usize] {
            self.program_counter += 2;
        }
    }

    // annn
    fn ldi(&mut self, opcode: Opcode) {
        self.index = opcode.nnn as usize;
    }

    // bnnn
    fn jp0(&mut self, opcode: Opcode) {
        self.program_counter = opcode.nnn as usize + self.registers[0] as usize - 2;
    }

    // cxnn
    fn rnd(&mut self, opcode: Opcode) {
        self.registers[opcode.x as usize] = rand::random::<u8>() & opcode.nn;
    }

    // dxyn
    fn drw(&mut self, opcode: Opcode) {
        self.has_drawn = true;
        self.has_handled_draw = false;
        self.registers[0xf] = 0;
        for i in 0..opcode.n {
            let y = self.registers[opcode.y as usize] + i;
            let sprite = self.memory[self.index + i as usize];
            let x = self.registers[opcode.x as usize];
            let x_byte = (x / 8) % 8;
            let y_offset = y % 32;

            let pixel_location = (x_byte + (y_offset * 8)) as usize;
            let shift_amount = x % 8;
            if self.screen[pixel_location] & (sprite >> shift_amount) != 0 {
                self.registers[0xf] = 1;
            }
            self.screen[pixel_location] ^= sprite >> shift_amount;

            let pixel_location = (((x_byte + 1) % 8) + (y_offset * 8)) as usize;
            let shift_amount = 8 - shift_amount;
            if shift_amount == 8 {
                continue;
            }
            if self.screen[pixel_location] & (sprite << shift_amount) != 0 {
                self.registers[0xf] = 1;
            }
            self.screen[pixel_location] ^= sprite << shift_amount;
        }
    }

    // ex9e
    fn skp(&mut self, opcode: Opcode) {
        if self.keys[self.registers[opcode.x as usize] as usize] {
            self.program_counter += 2;
        }
    }

    // exa1
    fn skpn(&mut self, opcode: Opcode) {
        if !self.keys[self.registers[opcode.x as usize] as usize] {
            self.program_counter += 2;
        }
    }

    // fx07
    fn ldxdt(&mut self, opcode: Opcode) {
        self.registers[opcode.x as usize] = self.delay;
    }

    // fx0a
    fn ldk(&mut self, opcode: Opcode) {
        let mut wait = true;

        for i in 0..=0xf {
            if self.keys[i] {
                wait = false;
                self.registers[opcode.x as usize] = i as u8;
                break;
            }
        }

        if wait {
            self.program_counter -= 2;
        }
    }

    // fx15
    fn lddt(&mut self, opcode: Opcode) {
        self.delay = self.registers[opcode.x as usize];
    }

    // fx18
    fn ldst(&mut self, opcode: Opcode) {
        self.sound = self.registers[opcode.x as usize];
    }

    // fx1e
    fn addi(&mut self, opcode: Opcode) {
        self.index += self.registers[opcode.x as usize] as usize;
    }

    // fx29
    fn ldf(&mut self, opcode: Opcode) {
        self.index = self.registers[opcode.x as usize] as usize * 5;
    }

    // fx33
    fn ldb(&mut self, opcode: Opcode) {
        self.memory[self.index] = self.registers[opcode.x as usize] / 100;
        self.memory[self.index + 1] = (self.registers[opcode.x as usize] / 10) % 10;
        self.memory[self.index + 2] = self.registers[opcode.x as usize] % 10;
    }

    // fx55
    fn ldix(&mut self, opcode: Opcode) {
        for i in 0..=opcode.x {
            self.memory[self.index + i as usize] = self.registers[i as usize];
        }
    }

    // fx65
    fn ldxi(&mut self, opcode: Opcode) {
        for i in 0..=opcode.x {
            self.registers[i as usize] = self.memory[self.index + i as usize];
        }
    }

    pub fn load(&mut self, rom: Vec<u8>) {
        for i in 0..rom.len() {
            self.memory[0x200 + i] = rom[i];
        }
    }
}
