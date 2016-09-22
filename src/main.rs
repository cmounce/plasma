extern crate sdl2;

mod colormapper;
mod fastmath;
mod formulas;
mod genetics;
mod gradient;
mod plasma;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::cmp;
use std::f32;
use std::time::SystemTime;
use plasma::Plasma;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

fn main() {
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let window = video.window("plasma", WIDTH, HEIGHT).build().unwrap();

    let mut renderer = window.renderer().build().unwrap();
    let mut plasma = Plasma::new(&mut renderer, WIDTH, HEIGHT);

    let mut running = true;
    let mut event_pump = sdl.event_pump().unwrap();
    let mut avg_render_time = 0.0;
    let mut avg_render_time_count = 0;
    while running {
        let timestamp = SystemTime::now();

        // Draw plasma, process events
        plasma.update(&mut renderer);
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => { running = false; break },
                Event::KeyDown { keycode: Some(keycode), ..} => {
                    match keycode {
                        Keycode::Equals | Keycode::Plus | Keycode::KpPlus => {
                            plasma.approve();
                        },
                        Keycode::Minus | Keycode::Underscore | Keycode::KpMinus => {
                            plasma.reject();
                        },
                        _ => ()
                    }
                },
                _ => ()
            }
        }

        // Manage time
        let duration = timestamp.elapsed().unwrap();
        let target_ms = 100;
        let actual_ms = (duration.subsec_nanos() as u64)/1000000 + duration.as_secs()*1000;
        avg_render_time += actual_ms as f32;
        avg_render_time_count += 1;
        if actual_ms > target_ms {
            println!("Target frame delay is {} but actual time taken is {}", target_ms, actual_ms);
        } else {
            std::thread::sleep(std::time::Duration::from_millis(target_ms - actual_ms));
        }
        plasma.add_time((cmp::max(target_ms, actual_ms) as f32)/1000.0);
    }
    println!("Average render time: {} ms", avg_render_time/(avg_render_time_count as f32));
}
