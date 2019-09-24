use crate::chip8::Chip8;
use crossterm::{cursor, input, terminal, AlternateScreen, InputEvent, KeyEvent};
use std::{
    fs::File,
    io::{stdout, Error, Read, Write},
    // ops::Add,
    time::{Duration, SystemTime},
};

enum Event {
    Quit,
}

pub struct App {
    chip8: Chip8,
}

impl App {
    pub fn new() -> Self {
        App {
            chip8: Chip8::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), Error> {
        let (terminal_starting_width, terminal_starting_height) = terminal().terminal_size();

        terminal().set_size(64, 32)?;
        let _screen = AlternateScreen::to_alternate(true);
        cursor().hide()?;

        let mut rom_file = File::open("roms/test_opcode.ch8")?;
        let mut rom: Vec<u8> = Vec::new();
        rom_file.read_to_end(&mut rom)?;
        self.chip8.load(rom);

        self.event_loop()?;

        terminal().set_size(
            terminal_starting_width as i16,
            terminal_starting_height as i16,
        )?;
        Ok(())
    }

    fn event_loop(&mut self) -> Result<(), Error> {
        let clock_duration = Duration::new(0, 1000000);
        // let clock_duration = time::Duration::new(0, 1000000000);
        let delay_duration = Duration::new(0, 16666667);

        let mut last_clock_time = SystemTime::now();
        let mut last_delay_time = last_clock_time;

        loop {
            match self.handle_input() {
                Some(event) => match event {
                    Event::Quit => break,
                },
                None => {}
            }

            // Clock cycle timer
            let mut duration = App::calculate_duration(last_clock_time);
            while duration >= clock_duration {
                self.chip8.clock();

                last_clock_time += clock_duration;
                duration = App::calculate_duration(last_clock_time);
            }

            // Delay and sound timer
            let mut duration = App::calculate_duration(last_delay_time);
            while duration >= delay_duration {
                self.chip8.delay = self.chip8.delay.saturating_sub(1);
                self.chip8.sound = self.chip8.sound.saturating_sub(1);
                for key in self.chip8.keys.iter_mut() {
                    *key = false;
                }
                self.draw()?;

                last_delay_time += delay_duration;
                duration = App::calculate_duration(last_delay_time);
            }
        }
        Ok(())
    }

    fn handle_input(&mut self) -> Option<Event> {
        let mut stdin = input().read_sync();

        while let Some(key_event) = stdin.next() {
            match key_event {
                InputEvent::Keyboard(event) => match event {
                    KeyEvent::Esc => return Some(Event::Quit),
                    KeyEvent::Char(c) => match c {
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

    fn draw(&mut self) -> Result<(), Error> {
        let mut stdout = stdout();
        if self.chip8.has_drawn && !self.chip8.has_handled_draw {
            self.chip8.has_handled_draw = true;

            for y in 0..self.chip8.screen_size.1 {
                cursor().goto(0, y as u16).unwrap();
                let mut line_buffer = String::new();

                for x in 0..self.chip8.screen_size.0 / 8 {
                    let pixel_block =
                        self.chip8.screen[(x + y * (self.chip8.screen_size.0 / 8)) as usize];

                    for i in 0..8 {
                        if (pixel_block << i) & 0b10000000 != 0 {
                            line_buffer.push('█');
                        } else {
                            line_buffer.push(' ');
                        }
                    }
                }
                write!(stdout, "{}", line_buffer)?;
            }
            stdout.flush()?;
        }
        Ok(())
    }

    fn calculate_duration(time_from: SystemTime) -> Duration {
        let now = SystemTime::now();
        match now.duration_since(time_from) {
            Ok(duration) => duration,
            Err(_) => Duration::new(0, 0),
        }
    }
}
