//! This is the relavant code for the Chip-8 interpreter.
//!
//! # About
//! So the Chip-8 virtual machine was designed by Joseph Weisbecker for the
//! COSMAC VIP and Telmac 1800 computers back in the 1970's.
//! Since then there has been an extension made to it, called Super Chip-8,
//! which isn't implemented in this project. There is also a discrepancy in how
//! a couple of the opcodes were used in some implementations, as a result some
//! roms may not work as intended.
//!
//! ## Memory
//! So the memory of Chip-8 is 4k bytes where the program starts at address `0x200`
//! before that address the hexadecimal characters are usually stored, in newer
//! interpreters, however in older interpreters, the entire interpreter would be
//! there.
//!
//! ## Registers
//! So there are 16 8-bit registers where register VF is used to flag certain
//! instructions as having done something like integer over/under flow and if there
//! was a collision in when drawing.
//!
//! ## Stack
//! So the stack in the original version allowed up to 24 levels of nesting,
//! however 16 is usually normal to implement afaik.
//!
//! ## Timers
//! There are two timers, which both run at a frequency of 60Hz. The way they
//! work is by ticking down until they reach zero. One timer is used for delay
//! events for the games, and the sound timer plays a sound until it hits 0.
//!
//! ## Input
//! The input for Chip-8 is based on a hex keypad which contains only hexadecimal
//! characters (0-9A-F) arranged in a 4x4 grid. In modern interpreters they get mapped as follows
//! ```
//! |1|2|3|c|    |1|2|3|4|
//! |4|5|6|d|    |q|w|e|r|
//! |7|8|9|e|    |a|s|d|f|
//! |a|0|b|f|    |z|x|c|v|
//! ```
//!
//! ## Graphics
//! The display resolution is 64x32 pixels, which are drawn to the screen with
//! sprites that are xor'ed to the screen buffer.

/// This is a helper struct, so that the opcodes can be parsed, and used more
/// easily
pub struct Opcode {
    code: u16,
    n: u8,
    nn: u8,
    nnn: u16,
    x: u8,
    y: u8,
}

impl Opcode {
    /// Parses the opcode from the 16-bit integer
    pub fn new(code: u16) -> Opcode {
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

/// This is my rendition of the interpreter
pub struct Chip8 {
    /// This is `V`
    pub registers: [u8; 16],
    /// This is `I`
    pub index: usize,
    /// This is the delay timer
    pub delay: u8,
    /// This is the sound timer
    pub sound: u8,
    /// This is `PC`
    pub program_counter: usize,
    /// This is `SP`
    pub stack_pointer: usize,
    pub stack: [usize; 16],
    pub memory: [u8; 0xfff],
    pub screen_size: (u8, u8),
    pub screen: Vec<u8>,
    /// This is to control which version of the instruction it should execute
    /// since there is a discrepancy in the documentation that people have been
    /// able to get their hands on, not being exactly the same
    pub other_mode: bool,
    /// This keeps track of which of the keys are down
    pub keys: [bool; 16],
    /// This keeps track if the interpreter has executed a draw command, and should
    /// not be updated outside of the interpreter
    pub has_drawn: bool,
    /// This keeps track if the parent program of the interpreter has handled it's draw
    pub has_handled_draw: bool,
}

/// This is to create a type for all of the instruction functions so that
/// a debugger can be attached to it, and be provided mnemonics
type Instruction = fn(&mut Chip8, &Opcode);

impl Chip8 {
    /// Creates a default Chip8 instance
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
        // resizes the screen to be 64x32 pixels wide
        chip8.screen.resize((64 / 8) * 32, 0);

        // The following mess is to add the letters that can be printed to the
        // screen, look at the hex value to know which character it represents
        chip8.add_letter(
            0x0,
            &[0b11110000, 0b10010000, 0b10010000, 0b10010000, 0b11110000],
        );
        chip8.add_letter(
            0x1,
            &[0b00100000, 0b01100000, 0b00100000, 0b00100000, 0b01110000],
        );
        chip8.add_letter(
            0x2,
            &[0b11110000, 0b00010000, 0b11110000, 0b10000000, 0b11110000],
        );
        chip8.add_letter(
            0x3,
            &[0b11110000, 0b00010000, 0b11110000, 0b00010000, 0b11110000],
        );
        chip8.add_letter(
            0x4,
            &[0b10010000, 0b10010000, 0b11110000, 0b00010000, 0b00010000],
        );
        chip8.add_letter(
            0x5,
            &[0b11110000, 0b10000000, 0b11110000, 0b00010000, 0b11110000],
        );
        chip8.add_letter(
            0x6,
            &[0b11110000, 0b10000000, 0b11110000, 0b10010000, 0b11110000],
        );
        chip8.add_letter(
            0x7,
            &[0b11110000, 0b00010000, 0b00100000, 0b01000000, 0b01000000],
        );
        chip8.add_letter(
            0x8,
            &[0b11110000, 0b10010000, 0b11110000, 0b10010000, 0b11110000],
        );
        chip8.add_letter(
            0x9,
            &[0b11110000, 0b10010000, 0b11110000, 0b00010000, 0b11110000],
        );
        chip8.add_letter(
            0xa,
            &[0b11110000, 0b10010000, 0b11110000, 0b10010000, 0b10010000],
        );
        chip8.add_letter(
            0xb,
            &[0b11100000, 0b10010000, 0b11100000, 0b10010000, 0b11100000],
        );
        chip8.add_letter(
            0xc,
            &[0b11110000, 0b10000000, 0b10000000, 0b10000000, 0b11110000],
        );
        chip8.add_letter(
            0xd,
            &[0b11100000, 0b10010000, 0b10010000, 0b10010000, 0b11100000],
        );
        chip8.add_letter(
            0xe,
            &[0b11110000, 0b10000000, 0b11110000, 0b10000000, 0b11110000],
        );
        chip8.add_letter(
            0xf,
            &[0b11110000, 0b10000000, 0b11110000, 0b10000000, 0b10000000],
        );

        chip8
    }

    /// A helper function that is used to add a letter to the beginning of the
    /// interpreter
    fn add_letter(&mut self, letter: usize, sprite: &[u8; 5]) {
        // Sets up the offset in memory for the letter to be placed in
        let offset: usize = letter * 5;
        // Loops through the sprite's size
        for i in 0 as usize..5 {
            // Places it in memory
            self.memory[offset + i] = sprite[i];
        }
    }

    /// This is where the interpreter runs all of the code it needs to
    pub fn clock(&mut self) {
        // Gets and parses the current opcode that needs to be ran
        let opcode = self.get_current_opcode();

        // If the parent application has handled the draw instruction set `has_drawn`
        // and `had_handled_draw` to false
        if self.has_handled_draw {
            self.has_drawn = false;
            self.has_handled_draw = false;
        }

        // Gets the associated function for the opcode, and runs the it
        self.get_instruction(&opcode)(self, &opcode);

        // Increments the program counter by one instruction or 2 bytes
        self.program_counter += 2;
    }

    /// Returns the parsed version of the opcode that needs to be ran
    fn get_current_opcode(&self) -> Opcode {
        let code = (self.memory[self.program_counter] as u16) << 8
            | self.memory[self.program_counter + 1] as u16;
        Opcode::new(code)
    }

    /// Returns the function for the opcode provided
    fn get_instruction(&self, opcode: &Opcode) -> Instruction {
        self.parse_opcode(&opcode).1
    }

    /// Gets the instruction relative to the current one, used for
    /// when the parent application wants to see which instruction is running.
    /// Used like so:
    /// ```rust
    /// fn do_stuff(chip8: &Chip8) {
    ///     chip8.get_relative_instruction(-2);
    ///     chip8.get_relative_instruction(-1);
    ///     chip8.get_relative_instruction(0);
    ///     chip8.get_relative_instruction(1);
    ///     chip8.get_relative_instruction(2);
    /// }
    /// ```
    pub fn get_relative_instruction(&self, relative: i32) -> &'static str {
        // gets the absolute value of the relative address
        let absolute = if relative < 0 { -relative } else { relative } as usize * 2;
        // adds or subtracts the relative address depending on whether it was negative or not
        let relative_address = if relative < 0 {
            self.program_counter - absolute
        } else {
            self.program_counter + absolute
        };

        // gets the opcode stored at that address
        let code =
            (self.memory[relative_address] as u16) << 8 | self.memory[relative_address + 1] as u16;
        // parse the opcode
        let opcode = Opcode::new(code);
        // return the mnemonic
        self.parse_opcode(&opcode).0
    }

    /// Parses the opcode and returns the corresponding function and mnemonic
    pub fn parse_opcode(&self, opcode: &Opcode) -> (&'static str, Instruction) {
        match opcode.code {
            0x00e0 => ("cls", Self::cls),
            0x00ee => ("ret", Self::ret),
            _ => match opcode.code >> 12 {
                0x1 => ("jp", Self::jp),
                0x2 => ("call", Self::call),
                0x3 => ("se", Self::se),
                0x4 => ("sne", Self::sne),
                0x5 => match opcode.code & 0xf {
                    0x0 => ("sey", Self::sey),
                    _ => ("nai", Self::nai),
                },
                0x6 => ("ld", Self::ld),
                0x7 => ("add", Self::add),
                0x8 => match opcode.code & 0xf {
                    0x0 => ("ldy", Self::ldy),
                    0x1 => ("or", Self::or),
                    0x2 => ("and", Self::and),
                    0x3 => ("xor", Self::xor),
                    0x4 => ("addy", Self::addy),
                    0x5 => ("sub", Self::sub),
                    0x6 => {
                        if self.other_mode {
                            ("shr", Self::shr)
                        } else {
                            ("shry", Self::shry)
                        }
                    }
                    0x7 => ("subn", Self::subn),
                    0xe => {
                        if self.other_mode {
                            ("shl", Self::shl)
                        } else {
                            ("shly", Self::shly)
                        }
                    }
                    _ => ("nai", Self::nai),
                },
                0x9 => match opcode.code & 0xf {
                    0x0 => ("sney", Self::sney),
                    _ => ("nai", Self::nai),
                },
                0xa => ("ldi", Self::ldi),
                0xb => ("jp0", Self::jp0),
                0xc => ("rnd", Self::rnd),
                0xd => ("drw", Self::drw),
                0xe => match opcode.code & 0xff {
                    0x9e => ("skp", Self::skp),
                    0xa1 => ("skpn", Self::skpn),
                    _ => ("nai", Self::nai),
                },
                0xf => match opcode.code & 0xff {
                    0x07 => ("ldxdt", Self::ldxdt),
                    0x0a => ("ldk", Self::ldk),
                    0x15 => ("lddt", Self::lddt),
                    0x18 => ("ldst", Self::ldst),
                    0x1e => ("addi", Self::addi),
                    0x29 => ("ldf", Self::ldf),
                    0x33 => ("ldb", Self::ldb),
                    0x55 => ("ldix", Self::ldix),
                    0x65 => ("ldxi", Self::ldxi),
                    _ => ("nai", Self::nai),
                },
                _ => ("nai", Self::nai),
            },
        }
    }

    /// Not an instruction, used to provide a mnemonic for when the interpreter
    /// tries to give a mnemonic for a piece of memory that is not actually
    /// an instruction.
    pub fn nai(&mut self, _opcode: &Opcode) {}

    /// Opcode: `00e0`
    ///
    /// Explanation: Clears the screen.
    fn cls(&mut self, _opcode: &Opcode) {
        self.has_drawn = true;
        for pixel in self.screen.iter_mut() {
            *pixel = 0;
        }
    }

    /// Opcode: `00ee`
    ///
    /// Explanation: Returns from a subroutine.
    fn ret(&mut self, _opcode: &Opcode) {
        self.program_counter = self.stack[self.stack_pointer];
        self.stack_pointer -= 1;
    }

    /// Opcode: `1nnn`
    ///
    /// Explanation: Jumps to address nnn.
    fn jp(&mut self, opcode: &Opcode) {
        self.program_counter = opcode.nnn as usize - 2;
    }

    /// Opcode: `2nnn`
    ///
    /// Explanation: Calls subroutine at nnn.
    fn call(&mut self, opcode: &Opcode) {
        self.stack_pointer += 1;
        self.stack[self.stack_pointer] = self.program_counter;
        self.program_counter = opcode.nnn as usize - 2;
    }

    /// Opcode: `3xnn`
    ///
    /// Explanation: Skips the next instruction if register x equals nn.
    fn se(&mut self, opcode: &Opcode) {
        if self.registers[opcode.x as usize] == opcode.nn {
            self.program_counter += 2;
        }
    }

    /// Opcode: `4xnn`
    ///
    /// Explanation: Skips the next instruction if register x doesn't equal nn.
    fn sne(&mut self, opcode: &Opcode) {
        if self.registers[opcode.x as usize] != opcode.nn {
            self.program_counter += 2;
        }
    }

    /// Opcode: `5xy0`
    ///
    /// Explanation: Skips the next instruction if register x equals register y.
    fn sey(&mut self, opcode: &Opcode) {
        if self.registers[opcode.x as usize] == self.registers[opcode.y as usize] {
            self.program_counter += 2;
        }
    }

    /// Opcode: `6xnn`
    ///
    /// Explanation: Sets register x to nn.
    fn ld(&mut self, opcode: &Opcode) {
        self.registers[opcode.x as usize] = opcode.nn;
    }

    /// Opcode: `7xnn`
    ///
    /// Explanation: Adds nn to register x without changing the carry flag.
    fn add(&mut self, opcode: &Opcode) {
        let x = &mut self.registers[opcode.x as usize];
        *x = x.wrapping_add(opcode.nn);
    }

    /// Opcode: `8xy0`
    ///
    /// Explanation: Sets register x to the value of register y.
    fn ldy(&mut self, opcode: &Opcode) {
        self.registers[opcode.x as usize] = self.registers[opcode.y as usize];
    }

    /// Opcode: `8xy1`
    ///
    /// Explanation: Sets register x to the value of the bitwise *or* of register x and register y.
    fn or(&mut self, opcode: &Opcode) {
        self.registers[opcode.x as usize] |= self.registers[opcode.y as usize];
    }

    /// Opcode: `8xy2`
    ///
    /// Explanation: Sets register x to the value of the bitwise *and* of register x and register y.
    fn and(&mut self, opcode: &Opcode) {
        self.registers[opcode.x as usize] &= self.registers[opcode.y as usize];
    }

    /// Opcode: `8xy3`
    ///
    /// Explanation: Sets register x to the value of the bitwise *xor* of register x and y.
    fn xor(&mut self, opcode: &Opcode) {
        self.registers[opcode.x as usize] ^= self.registers[opcode.y as usize];
    }

    /// Opcode: `8xy4`
    ///
    /// Explanation: Adds register y to register x, and sets register f to 1 if there is an overflow, and 0 if there isn't.
    fn addy(&mut self, opcode: &Opcode) {
        self.registers[0xf] = 0;
        let result =
            self.registers[opcode.x as usize].overflowing_add(self.registers[opcode.y as usize]);
        self.registers[opcode.x as usize] = result.0;
        if result.1 {
            self.registers[0xf] = 1;
        }
    }

    /// Opcode: `8xy5`
    ///
    /// Explanation: Subtracts register y from register x and sets register f to 1 if there is an under flow, and 0 if there isn't.
    fn sub(&mut self, opcode: &Opcode) {
        self.registers[0xf] = 0;
        let result =
            self.registers[opcode.x as usize].overflowing_sub(self.registers[opcode.y as usize]);
        self.registers[opcode.x as usize] = result.0;
        if result.1 {
            self.registers[0xf] = 1;
        }
    }

    /// Opcode: `8x06`
    ///
    /// Explanation: Stores the least significant bit of register x into register f and shifts register x by 1.
    ///
    /// Note: This is one of the functions whose definition has changed over the years. This is the default.
    fn shr(&mut self, opcode: &Opcode) {
        self.registers[0xf] = 0;
        if self.registers[opcode.x as usize] & 0b1 == 1 {
            self.registers[0xf] = 1;
        }
        self.registers[opcode.x as usize] = self.registers[opcode.x as usize] >> 1;
    }

    /// Opcode: `8xy6`
    ///
    /// Explanation: Stores the least significant bit of register x into register f and shifts register x by the value of register y.
    ///
    /// Note: This is one of the functions whose definition has changed over the years. This is used if other_mode is set to true.
    fn shry(&mut self, opcode: &Opcode) {
        self.registers[0xf] = 0;
        if self.registers[opcode.y as usize] & 0b1 == 1 {
            self.registers[0xf] = 1;
        }
        self.registers[opcode.x as usize] = self.registers[opcode.y as usize] >> 1;
    }

    /// Opcode: `8xy7`
    ///
    /// Explanation: Sets register x to register y minus register x, setting register f to 1 if there is an underflow, and 0 if there isn't.
    fn subn(&mut self, opcode: &Opcode) {
        self.registers[0xf] = 0;
        let result =
            self.registers[opcode.y as usize].overflowing_sub(self.registers[opcode.x as usize]);
        self.registers[opcode.x as usize] = result.0;
        if result.1 {
            self.registers[0xf] = 1;
        }
    }

    /// Opcode: `8x0e`
    ///
    /// Explanation: Stores the most significant bit of register x into register f then shifts register x by 1.
    ///
    /// Note: This is one of the functions whose definition has changed over the years. This is the default.
    fn shl(&mut self, opcode: &Opcode) {
        self.registers[0xf] = 0;
        if self.registers[opcode.x as usize] & 0b10000000 != 0 {
            self.registers[0xf] = 1;
        }
        self.registers[opcode.x as usize] = self.registers[opcode.x as usize] << 1;
    }

    /// Opcode: `8xye`
    ///
    /// Explanation: Stores the most significant bit of register x into register f then shifts register x by the value in register y.
    ///
    /// Note: This is one of the functions whose definition has changed over the years. This is used if other_mode is set to true.
    fn shly(&mut self, opcode: &Opcode) {
        self.registers[0xf] = 0;
        if self.registers[opcode.y as usize] & 0b10000000 != 0 {
            self.registers[0xf] = 1;
        }
        self.registers[opcode.x as usize] = self.registers[opcode.y as usize] << 1;
    }

    /// Opcode: `9xy0`
    ///
    /// Explanation: skips the next instruction if register x doesn't equal register y.
    fn sney(&mut self, opcode: &Opcode) {
        if self.registers[opcode.x as usize] != self.registers[opcode.y as usize] {
            self.program_counter += 2;
        }
    }

    /// Opcode: `annn`
    ///
    /// Explanation: Sets the index to address nnn.
    fn ldi(&mut self, opcode: &Opcode) {
        self.index = opcode.nnn as usize;
    }

    /// Opcode: `bnnn`
    ///
    /// Explanation: Jumps to address nnn plus the value of register 0.
    fn jp0(&mut self, opcode: &Opcode) {
        self.program_counter = opcode.nnn as usize + self.registers[0] as usize - 2;
    }

    /// Opcode: `cxnn`
    ///
    /// Explanation: Sets register x to the bitwise and of a random number and nn.
    fn rnd(&mut self, opcode: &Opcode) {
        self.registers[opcode.x as usize] = rand::random::<u8>() & opcode.nn;
    }

    /// Opcode: `dxyn`
    ///
    /// Explanation: Draws a sprite at coordinates located in registers x and y with a width of 8 pixels and a height of n pixels.
    /// The sprite it reads is the one pointed to by index and if any pixels are changed from 1 to 0, sets register f to 1, otherwise 0.
    fn drw(&mut self, opcode: &Opcode) {
        self.has_drawn = true;
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

    /// Opcode: `ex9e`
    ///
    /// Explanation: Skips the next instruction if the key stored in register x is pressed.
    fn skp(&mut self, opcode: &Opcode) {
        if self.keys[self.registers[opcode.x as usize] as usize] {
            self.program_counter += 2;
        }
    }

    /// Opcode: `exa1`
    ///
    /// Explanation: Skips the next instruction if the key stored in register x is not pressed.
    fn skpn(&mut self, opcode: &Opcode) {
        if !self.keys[self.registers[opcode.x as usize] as usize] {
            self.program_counter += 2;
        }
    }

    /// Opcode: `fx07`
    ///
    /// Explanation: Sets register x to the value of the delay timer.
    fn ldxdt(&mut self, opcode: &Opcode) {
        self.registers[opcode.x as usize] = self.delay;
    }

    /// Opcode: `fx0a`
    ///
    /// Explanation: Waits for a key to be pressed, then stores that value into register x.
    ///
    /// Note: This operation blocks all other execution.
    fn ldk(&mut self, opcode: &Opcode) {
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

    /// Opcode: `fx15`
    ///
    /// Explanation: Sets the delay timer to the value of register x.
    fn lddt(&mut self, opcode: &Opcode) {
        self.delay = self.registers[opcode.x as usize];
    }

    /// Opcode: `fx18`
    ///
    /// Explanation: Sets the sound timer to the value of register x.
    fn ldst(&mut self, opcode: &Opcode) {
        self.sound = self.registers[opcode.x as usize];
    }

    /// Opcode: `fx1e`
    ///
    /// Explanation: Adds the value of register x to the index.
    fn addi(&mut self, opcode: &Opcode) {
        self.index += self.registers[opcode.x as usize] as usize;
    }

    /// Opcode: `fx29`
    ///
    /// Explanation: Sets the index to the location for the character stored in register x.
    ///
    /// Note: This is represented by a 4x5 pixel font.
    fn ldf(&mut self, opcode: &Opcode) {
        self.index = self.registers[opcode.x as usize] as usize * 5;
    }

    /// Opcode: `fx33`
    ///
    /// Explanation: Stores the binary coded decimal representation of the value
    /// in register x with the most significant number stored at the index, and
    /// the least significant number stored at the index + 2.
    fn ldb(&mut self, opcode: &Opcode) {
        self.memory[self.index] = self.registers[opcode.x as usize] / 100;
        self.memory[self.index + 1] = (self.registers[opcode.x as usize] / 10) % 10;
        self.memory[self.index + 2] = self.registers[opcode.x as usize] % 10;
    }

    /// Opcode: `fx55`
    ///
    /// Explanation: Stores register 0 through register x into memory starting at
    /// the index, without modifying the index.
    fn ldix(&mut self, opcode: &Opcode) {
        for i in 0..=opcode.x {
            self.memory[self.index + i as usize] = self.registers[i as usize];
        }
    }

    /// Opcode: `fx65`
    ///
    /// Explanation: Loads register 0 through register x with values from memory
    /// starting at the index, without modifying the index.
    fn ldxi(&mut self, opcode: &Opcode) {
        for i in 0..=opcode.x {
            self.registers[i as usize] = self.memory[self.index + i as usize];
        }
    }

    /// Loads the bytes of the rom into the memory starting at location `0x200`.
    pub fn load(&mut self, rom: Vec<u8>) {
        for i in 0..rom.len() {
            self.memory[0x200 + i] = rom[i];
        }
    }
}
