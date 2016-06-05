extern crate sdl2;

use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::f32;

fn main() {
    println!("Hello, world!");
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let window = video.window("plasma", 640, 480).build().unwrap();
    
    let mut renderer = window.renderer().build().unwrap();
    for x in 0..640 {
        for y in 0..480 {
            let r = x/4;
            let g = y/4;
            let b = ((x as f32)/20.0).cos()*127.0 + 127.0;
            renderer.set_draw_color(Color::RGB(r as u8, g as u8, b as u8));
            renderer.draw_point(Point::new(x as i32, y as i32)).unwrap();
        }
    }
    renderer.present();
    
    let mut event_pump = sdl.event_pump().unwrap();
    loop {
        let event = event_pump.wait_event();
        match event {
            Event::Quit {timestamp:t} => {println!("{:?}", t); break},
            _ => ()
        }
        //println!("{:?}", event);
    }
    //std::thread::sleep(std::time::Duration::new(5, 0));
}
