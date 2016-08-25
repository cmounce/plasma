extern crate sdl2;

mod fastmath;
mod colormapper;
mod genetics;
mod gradient;

use fastmath::FastMath;
use colormapper::ColorMapper;
use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Renderer;
use sdl2::render::Texture;
use std::cmp;
use std::f32;
use std::time::SystemTime;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

struct Plasma {
    color_mapper: ColorMapper,
    texture: Texture,
    pixel_data: Vec<u8>,
    time: f32
}

impl Plasma {
    fn new(renderer: &mut Renderer) -> Plasma {
        Plasma {
            color_mapper: ColorMapper::new(),
            texture: renderer.create_texture_streaming(PixelFormatEnum::RGB24, WIDTH, HEIGHT).unwrap(),
            pixel_data: vec![0; (WIDTH*HEIGHT*3) as usize],
            time: 0.0
        }
    }

    fn plot(&mut self, x: u32, y: u32, red: u8, green: u8, blue: u8) {
        let offset = ((x + y*WIDTH)*3) as usize;
        self.pixel_data[offset] = red;
        self.pixel_data[offset + 1] = green;
        self.pixel_data[offset + 2] = blue;
    }

    fn calculate_value(&self, x: f32, y: f32) -> f32 {
        let x_adj = x*200.0;
        let y_adj = y*200.0;

        let mut value = 0.0;
        value += ((x_adj/23.0 + self.time)/10.0).wave();
        value += ((x_adj/13.0 + (y_adj/17.0)*(self.time/20.0).wave() )/10.0).wave();
        let dx = (self.time/19.0).wave()*75.0 + 100.0 - x_adj;
        let dy = (self.time/31.0 + 0.5).wave()*75.0 + 100.0 - y_adj;
        value += ((dx*dx + dy*dy).sqrt()/290.0 + self.time/10.0).wave();
        return value;
    }

    fn calculate_color(&self, x: f32, y: f32) -> (u8, u8, u8) {
        self.color_mapper.convert(self.calculate_value(x, y))
    }

    fn update(&mut self, renderer: &mut Renderer) {
        let scale = 1.0/((WIDTH as f32).min(HEIGHT as f32));
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let (r, g, b) = self.calculate_color((x as f32)*scale, (y as f32)*scale);
                self.plot(x, y, r as u8, g as u8, b as u8);
            }
        }
        self.texture.update(None, &self.pixel_data[..], (WIDTH*3) as usize).unwrap();
        renderer.copy(&self.texture, None, None);
        renderer.present();
    }

    fn add_time(&mut self, time: f32) {
        self.time += time;
    }
}

fn main() {
    println!("Hello, world!");
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let window = video.window("plasma", WIDTH, HEIGHT).build().unwrap();

    let mut renderer = window.renderer().build().unwrap();
    let mut plasma = Plasma::new(&mut renderer);

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
                Event::Quit {..} => {running = false; break},
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
