mod chip8;

use crossterm::{cursor, terminal, ClearType};
use crossterm_input::{input, InputEvent, KeyEvent, RawScreen};
use std::fs::File;
use std::io::prelude::*;
use std::time;

fn main() {
    let terminal = terminal();
    terminal.clear(ClearType::All).unwrap();
    terminal.set_size(64, 32).unwrap();
    let cursor = cursor();
    cursor.hide().unwrap();
    let _screen = RawScreen::into_raw_mode();

    let input = input();

    let mut chip8 = chip8::Chip8::new();

    let mut rom = match File::open("roms/output.ch8") {
        Ok(r) => r,
        Err(_) => panic!("Could not open rom"),
    };
    let mut rom_bytes: Vec<u8> = Vec::new();
    rom.read_to_end(&mut rom_bytes).unwrap();

    chip8.load(rom_bytes);

    let clock_duration = time::Duration::new(0, 1000000);
    let delay_duration = time::Duration::new(0, 16666667);

    let mut last_clock_time = time::SystemTime::now();
    let mut last_delay_time = last_clock_time;

    loop {
        let now = time::SystemTime::now();
        let duration = match now.duration_since(last_clock_time) {
            Ok(duration) => duration,
            Err(_) => panic!("last_clock_time was later than now."),
        };

        let mut stdin = input.read_sync();

        if let Some(key_event) = stdin.next() {
            match key_event {
                InputEvent::Keyboard(event) => match event {
                    KeyEvent::Esc => return,
                    KeyEvent::Char(c) => match c {
                        '1' => chip8.keys[0x1] = true,
                        '2' => chip8.keys[0x2] = true,
                        '3' => chip8.keys[0x3] = true,
                        '4' => chip8.keys[0xc] = true,
                        'q' => chip8.keys[0x4] = true,
                        'w' => chip8.keys[0x5] = true,
                        'e' => chip8.keys[0x6] = true,
                        'r' => chip8.keys[0xd] = true,
                        'a' => chip8.keys[0x7] = true,
                        's' => chip8.keys[0x8] = true,
                        'd' => chip8.keys[0x9] = true,
                        'f' => chip8.keys[0xe] = true,
                        'z' => chip8.keys[0xa] = true,
                        'x' => chip8.keys[0x0] = true,
                        'c' => chip8.keys[0xb] = true,
                        'v' => chip8.keys[0xf] = true,
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            }
        }

        if duration >= clock_duration {
            chip8.clock();
            last_clock_time = match last_clock_time.checked_add(clock_duration) {
                Some(time) => time,
                None => last_clock_time,
            };
        }

        let duration = match now.duration_since(last_delay_time) {
            Ok(duration) => duration,
            Err(_) => panic!("last_delay_time was later than now."),
        };

        if duration >= delay_duration {
            chip8.delay = chip8.delay.saturating_sub(1);
            chip8.sound = chip8.sound.saturating_sub(1);
            for key in chip8.keys.iter_mut() {
                *key = false;
            }
            last_delay_time = match last_delay_time.checked_add(delay_duration) {
                Some(time) => time,
                None => last_delay_time,
            }
        }

        if chip8.has_drawn {
            chip8.has_drawn = false;
            for y in 0..chip8.screen_size.1 {
                for x in 0..chip8.screen_size.0 / 8 {
                    cursor.goto((x * 8) as u16, y as u16).unwrap();
                    let pixel = chip8.screen[(x + y * (chip8.screen_size.0 / 8)) as usize];
                    let mut a = String::new();
                    for i in 0..8 {
                        if (pixel << i) & 0b10000000 != 0 {
                            a.push('█');
                        // print!("{} ", Colored::Bg(Color::Red));
                        } else {
                            a.push(' ');
                            // print!("{} ", Colored::Bg(Color::Black));
                        }
                    }
                    print!("{}", a);
                }
            }
        }
    }
}
