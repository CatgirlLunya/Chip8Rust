mod chip;
extern crate sdl2;

use std::fs;
use std::time::Duration;
use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use crate::chip::Chip;

const SCALE: u32 = 20;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Please supply a rom file!");
        return;
    }

    let sdl = sdl2::init().unwrap();
    let video_system = sdl.video().unwrap();
    let window = video_system.window("Chip8", 64*SCALE, 32*SCALE)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.clear();
    canvas.present();

    let mut chip = Chip::new(&canvas);
    chip.speed = 720;
    chip.load_rom(fs::read(&args[1]).unwrap());

    let mut event_pump = sdl.event_pump().unwrap();

    let frame_time = Duration::new(0, 1_000_000_000_u32 / chip.speed);
    let mut frames_passed = 1;
    let mut inputs_stored: Vec<Scancode> = Vec::new();
    let input_map = Chip::get_input_map();
    'game_loop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit{ .. } => break 'game_loop,
                Event::KeyDown {scancode, .. } => {
                    let code = scancode.unwrap();
                    if input_map.contains_key(&code) {
                        chip.input[input_map[&code] as usize] = true;
                        if chip.seeking_input {
                            inputs_stored.push(code);
                        }
                    }
                }
                Event::KeyUp {scancode, ..} => {
                    let code = scancode.unwrap();
                    if input_map.contains_key(&code) {
                        chip.input[input_map[&code] as usize] = false;
                        if chip.seeking_input {
                            if inputs_stored.contains(&code) {
                                chip.key_pressed = code;
                                inputs_stored.clear();
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        chip.update();
        if frames_passed == 12 {
            frames_passed = 0;
            chip.dec_timers();
        }
        chip.render(&mut canvas);
        frames_passed += 1;
        ::std::thread::sleep(frame_time);
    }
}
