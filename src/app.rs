//! This module contains all of the application relevant code that interacts
//! with the chip8 interpreter

use crate::chip8::Chip8;
use crossterm::{cursor, input, terminal, AlternateScreen, InputEvent, KeyEvent};
use std::{
    fs::File,
    io::{stdout, Error, Read, Write},
    time::{Duration, SystemTime},
};

/// Just an enum to check for events that the application needs to take care of
enum Event {
    Quit,
}

/// A struct that contains application-wide state
pub struct App {
    chip8: Chip8,
}

impl App {
    /// Creates a default App struct
    pub fn new() -> Self {
        App {
            chip8: Chip8::new(),
        }
    }

    /// Sets up the initial state for the app and calls the event loop
    pub fn run(&mut self) -> Result<(), Error> {
        // Get the current terminal's size, so that it can be restored when the application quits.
        let (terminal_starting_width, terminal_starting_height) = terminal().terminal_size();

        // Sets the terminal to the chip8 specification's size
        terminal().set_size(64, 32)?;
        // Creates an alternate screen, so that the contents of the terminal aren't
        // overridden
        let _screen = AlternateScreen::to_alternate(true);
        // hides the cursor
        // Note: doesn't work on Windows with using AlternateScreen
        cursor().hide()?;

        // Opens the rom file
        // Todo: This is hard coded, needs to be an option that is passed in
        let mut rom_file = File::open("roms/test_opcode.ch8")?;
        // Creates a buffer to store the file
        let mut rom: Vec<u8> = Vec::new();
        // Writes to the buffer
        rom_file.read_to_end(&mut rom)?;
        // Loads the rom into the interpreter's memory
        self.chip8.load(rom);

        // Runs the event loop, and stores the value in case if it throws an error
        let event_loop_result = self.event_loop();

        // Restore's the terminal's size to what it was before the application started
        terminal().set_size(
            terminal_starting_width as i16,
            terminal_starting_height as i16,
        )?;

        // Returns the result that was return from the event loop
        event_loop_result
    }

    /// This runs the chip8 interpreter, keeping track of the two different clocks
    /// that the interpreter needs
    fn event_loop(&mut self) -> Result<(), Error> {
        // It is hard to find the speed that the interpreter runs, but according
        // to a document I had read, it said that the computer that it was based
        // off of had a clock speed of 1KHz
        let clock_duration = Duration::new(0, 1000000);
        // The delays for the interpreter are ticked down at a rate of 60Hz
        let delay_duration = Duration::new(0, 16666667);

        // Sets the initial system time for the timers
        let mut last_clock_time = SystemTime::now();
        let mut last_delay_time = last_clock_time;

        // And now to the loop
        loop {
            // handle_input returns an Option<Event> so that if the user decides
            // to quit the application, they can
            match self.handle_input() {
                Some(event) => match event {
                    Event::Quit => break,
                },
                None => {}
            }

            // The duration since the last clock cycle
            let mut duration = App::calculate_duration(last_clock_time);
            // Keep running until the interpreter catches up it's clock cycles
            while duration >= clock_duration {
                // runs the current instruction
                self.chip8.clock();

                // adds the clock duration of the interpreter
                last_clock_time += clock_duration;
                // recalculate the duration to be re-checked
                duration = App::calculate_duration(last_clock_time);
            }

            // The duration since the last delay cycle
            let mut duration = App::calculate_duration(last_delay_time);
            // Keep running until the interpreter catches up the delay/sound timers
            while duration >= delay_duration {
                // The delay and sound timers tick down one every 1/60th of a second
                // until they hit 0
                self.chip8.delay_timer = self.chip8.delay_timer.saturating_sub(1);
                self.chip8.sound = self.chip8.sound.saturating_sub(1);
                // Sets all of the keys to be unpressed
                for key in self.chip8.keys.iter_mut() {
                    *key = false;
                }
                // Draws the interpreter's buffer, I believe that the screen that
                // the telemac updated at was 1/60th of a second, even if it is not,
                // it seems like a reasonable speed to update the screen
                self.draw()?;

                // basically the same thing as the clock duration/delay
                last_delay_time += delay_duration;
                duration = App::calculate_duration(last_delay_time);
            }
        }
        // Yay, nothing broke
        Ok(())
    }

    /// Sets the keys that are pressed, and handles sending the quit event
    fn handle_input(&mut self) -> Option<Event> {
        // Gets stdin, so that the key events can be checked
        let mut stdin = input().read_sync();

        // Iterates over every event that has passed
        while let Some(key_event) = stdin.next() {
            match key_event {
                InputEvent::Keyboard(event) => match event {
                    // There is no specific instruction for chip8 to quit the
                    // the program, so it has to be implemented in the interpreter
                    KeyEvent::Esc => return Some(Event::Quit),
                    KeyEvent::Char(c) => match c {
                        // The chip8 virtual computer was originally made for a
                        // computer that had a keypad using hexadecimal digits
                        // which is usually mapped in this way:
                        /*
                        123c    1234
                        456d    qwer
                        789e    asdf
                        a0bf    zxcv
                        */
                        '1' => self.chip8.keys[0x1] = true,
                        '2' => self.chip8.keys[0x2] = true,
                        '3' => self.chip8.keys[0x3] = true,
                        '4' => self.chip8.keys[0xc] = true,
                        'q' => self.chip8.keys[0x4] = true,
                        'w' => self.chip8.keys[0x5] = true,
                        'e' => self.chip8.keys[0x6] = true,
                        'r' => self.chip8.keys[0xd] = true,
                        'a' => self.chip8.keys[0x7] = true,
                        's' => self.chip8.keys[0x8] = true,
                        'd' => self.chip8.keys[0x9] = true,
                        'f' => self.chip8.keys[0xe] = true,
                        'z' => self.chip8.keys[0xa] = true,
                        'x' => self.chip8.keys[0x0] = true,
                        'c' => self.chip8.keys[0xb] = true,
                        'v' => self.chip8.keys[0xf] = true,
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            }
        }
        None
    }

    /// Prints out the chip8 interpreter's draw buffer to the terminal
    fn draw(&mut self) -> Result<(), Error> {
        let mut stdout = stdout();

        // this ensures that we don't draw to the terminal unless if the chip8
        // interpreter has drawn or cleared.
        if self.chip8.has_drawn && !self.chip8.has_handled_draw {
            self.chip8.has_handled_draw = true;

            // Iterate over each y coordinate by values of one
            for y in 0..self.chip8.screen_size.1 {
                // set the cursor to the left most column on the corresponding y coordinate
                cursor().goto(0, y as u16).unwrap();
                // create a buffer for each line that will be outputted to the terminal
                let mut line_buffer = String::new();

                // Iterate over each x coordinate by a factor of 1/8 because
                // of the amount of bits in use
                for x in 0..self.chip8.screen_size.0 / 8 {
                    // Get the u8 block of pixels to be drawn
                    let pixel_block =
                        self.chip8.screen[(x + y * (self.chip8.screen_size.0 / 8)) as usize];

                    // Iterate over each bit
                    for i in 0..8 {
                        // Move the corresponding pixel bit to the left most column,
                        // and check to see if it is on
                        if (pixel_block << i) & 0b10000000 != 0 {
                            // If the pixel is on, then push a fill block character
                            // (which is 3 bytes long apparently) to the line buffer
                            line_buffer.push('█');
                        } else {
                            // If it is off, push an empty block (space) to the line buffer
                            line_buffer.push(' ');
                        }
                    }
                }
                // Write the line to the terminal
                write!(stdout, "{}", line_buffer)?;
            }
            // Flush the content that has been written to the terminal
            stdout.flush()?;
        }
        // If we got here, then everything worked as intended
        Ok(())
    }

    // This is just a helper function, going into the semantic compression theory
    // being, if you use it more than once, make it into a function
    fn calculate_duration(time_from: SystemTime) -> Duration {
        // Get the current time
        let now = SystemTime::now();
        // Get the duration, and check to see if it makes sense/throws an error
        match now.duration_since(time_from) {
            Ok(duration) => duration,      // The duration is reasonable
            Err(_) => Duration::new(0, 0), // The duration is negative
        }
    }
}
