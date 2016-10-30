extern crate sdl2;

mod colormapper;
mod fastmath;
mod formulas;
mod genetics;
mod gradient;
mod plasma;
mod renderer;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::event::WindowEventId;
use std::f32;
use std::time::SystemTime;
use plasma::Plasma;

const FRAMES_PER_SECOND: f32 = 16.0;

// Default window size
const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

fn main() {
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let window = video.window("plasma", WIDTH, HEIGHT).resizable().build().unwrap();

    let mut renderer = window.renderer().build().unwrap();
    let mut plasma = Plasma::new(&mut renderer, WIDTH, HEIGHT);

    let mut running = true;
    let mut event_pump = sdl.event_pump().unwrap();
    let mut avg_render_time = 0.0;
    let mut avg_render_time_count = 0;
    while running {
        let timestamp = SystemTime::now();

        // Process events, draw plasma
        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown { keycode: Some(keycode), ..} => {
                    match keycode {
                        Keycode::Equals | Keycode::Plus | Keycode::KpPlus => {
                            plasma.approve();
                        },
                        Keycode::Minus | Keycode::Underscore | Keycode::KpMinus => {
                            plasma.reject();
                        },
                        Keycode::E => {
                            plasma.export_current_genome();
                        },
                        _ => ()
                    }
                },

                Event::Window {
                    win_event_id: WindowEventId::Resized,
                    data1: width,
                    data2: height, ..
                } => {
                    plasma.resize(&mut renderer, width as u32, height as u32);
                },

                Event::Quit {..} => { running = false; break },
                _ => ()
            }
        }
        plasma.update(&mut renderer);

        // Sleep to hit framerate
        let duration = timestamp.elapsed().unwrap();
        let target_ms = 1000.0/FRAMES_PER_SECOND;
        let actual_ms = duration.subsec_nanos() as f32/1_000_000.0 +
            duration.as_secs() as f32*1000.0;
        if actual_ms > target_ms {
            println!("Target frame delay is {} but actual time taken is {}", target_ms, actual_ms);
        } else {
            let sleep_ms = (target_ms - actual_ms).max(0.0) as u64;
            std::thread::sleep(std::time::Duration::from_millis(sleep_ms));
        }
        plasma.add_time(target_ms.max(actual_ms)/1000.0);

        // Calculate time statistics
        avg_render_time += actual_ms;
        avg_render_time_count += 1;
        if avg_render_time_count >= 50 {
            println!("Average render time: {} ms", avg_render_time/(avg_render_time_count as f32));
            avg_render_time = 0.0;
            avg_render_time_count = 0;
        }
    }
    if avg_render_time_count > 0 {
        println!("Average render time: {} ms", avg_render_time/(avg_render_time_count as f32));
    }
}
