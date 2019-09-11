mod chip8;

use crossterm::{cursor, terminal, ClearType};
use crossterm_input::{input, InputEvent, KeyEvent, RawScreen};
use std::fs::File;
use std::io::prelude::*;
use std::time;

fn init(chip8: &mut chip8::Chip8) {
    let terminal = terminal();
    terminal.clear(ClearType::All).unwrap();
    terminal.set_size(64, 32).unwrap();
    let cursor = cursor();
    cursor.hide().unwrap();
    let _screen = RawScreen::into_raw_mode();

    let mut rom = match File::open("roms/test_opcode.ch8") {
        Ok(r) => r,
        Err(_) => panic!("Could not open rom"),
    };
    let mut rom_bytes: Vec<u8> = Vec::new();
    rom.read_to_end(&mut rom_bytes).unwrap();

    chip8.load(rom_bytes);
}

fn draw(chip8: &mut chip8::Chip8) {
    if chip8.has_drawn && !chip8.has_handled_draw {
        let cursor = cursor();
        chip8.has_handled_draw;
        for y in 0..chip8.screen_size.1 {
            cursor.goto(0, y as u16).unwrap();
            let mut a = String::new();
            for x in 0..chip8.screen_size.0 / 8 {
                let pixel = chip8.screen[(x + y * (chip8.screen_size.0 / 8)) as usize];
                for i in 0..8 {
                    if (pixel << i) & 0b10000000 != 0 {
                        a.push('█');
                    } else {
                        a.push(' ');
                    }
                }
            }
            print!("{}", a);
        }
    }
}

fn handle_input(chip8: &mut chip8::Chip8) {
    let input = input();
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
}

fn calculate_duration(time_from: time::SystemTime) -> time::Duration {
    let now = time::SystemTime::now();
    let duration = match now.duration_since(time_from) {
        Ok(duration) => duration,
        Err(_) => time::Duration::new(0, 0),
    };
    duration
}

fn main() {
    let mut chip8 = chip8::Chip8::new();
    init(&mut chip8);

    let clock_duration = time::Duration::new(0, 1000000);
    let delay_duration = time::Duration::new(0, 16666667);

    let mut last_clock_time = time::SystemTime::now();
    let mut last_delay_time = last_clock_time;

    loop {
        handle_input(&mut chip8);

        let mut duration = calculate_duration(last_clock_time);

        while duration >= clock_duration {
            chip8.clock();
            last_clock_time = last_clock_time
                .checked_add(clock_duration)
                .unwrap_or(last_clock_time);
            duration = calculate_duration(last_clock_time);
        }

        duration = calculate_duration(last_delay_time);

        while duration >= delay_duration {
            chip8.delay = chip8.delay.saturating_sub(1);
            chip8.sound = chip8.sound.saturating_sub(1);
            draw(&mut chip8);
            for key in chip8.keys.iter_mut() {
                *key = false;
            }
            last_delay_time = last_delay_time
                .checked_add(delay_duration)
                .unwrap_or(last_delay_time);
            duration = calculate_duration(last_delay_time);
        }
    }
}
