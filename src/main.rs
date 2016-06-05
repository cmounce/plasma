extern crate sdl2;

use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Renderer;
use std::f32;
use std::time::SystemTime;

fn draw(renderer: &mut Renderer, time: f32) {
    for x in 0..640 {
        for y in 0..480 {
            let r = x/4;
            let g = y/4;
            let b = (((x as f32) + time*10.0)/20.0).cos()*127.0 + 127.0;
            renderer.set_draw_color(Color::RGB(r as u8, g as u8, b as u8));
            renderer.draw_point(Point::new(x as i32, y as i32)).unwrap();
        }
    }
    renderer.present();
}

fn main() {
    println!("Hello, world!");
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let window = video.window("plasma", 640, 480).build().unwrap();

    let mut renderer = window.renderer().build().unwrap();
    let mut time = 0.0;

    let mut running = true;
    let mut event_pump = sdl.event_pump().unwrap();
    while running {
        let timestamp = SystemTime::now();

        draw(&mut renderer, time);
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {running = false; break},
                _ => ()
            }
        }

        let duration = timestamp.elapsed().unwrap();
        let millis = (duration.subsec_nanos() as u64)/1000000 + duration.as_secs()*1000;
        let delay = 100;
        if delay < millis {
            println!("Frame delay is {} but actual time taken is {}", delay, millis);
        } else {
            std::thread::sleep(std::time::Duration::from_millis(delay - millis));
        }
        time += (delay as f32)/1000.0;
    }
}
