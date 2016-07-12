extern crate sdl2;

use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Renderer;
use sdl2::render::Texture;
use std::cmp;
use std::f32;
use std::time::SystemTime;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

trait FastMath<F> {
    fn xwave(&self) -> F;
    fn ywave(&self) -> F;
}

impl FastMath<f32> for f32 {
    // Like sin(), except its period is 10 instead of 2*PI
    fn ywave(&self) -> f32 {
        // x loops over the range (-0.5, 0.5)
        let adj = self/10.0;
        let x = adj - adj.floor() - 0.5;

        // f(x) = 16x^2 - 8x, a parabola centered on the interval (0.0, 0.5).
        // Having one x and one x.abs() flips the parabola when x is negative.
        x * x.abs().mul_add(16.0, -8.0)
    }

    // Like cos(), except its period is 10 instead of 2*PI
    fn xwave(&self) -> f32 {
        return (self + 2.5).ywave();
    }
}

macro_rules! assert_feq {
    ($expected:expr, $actual:expr) => (
        {
            let expected:f32 = $expected;
            let actual:f32 = $actual;
            assert!(expected.abs_sub(actual) < 0.01, "assertion failed: {} != {}", expected, actual);
        }
    );
}

#[test]
fn test_ywave() {
    // Test area around 0
    assert_feq!((-0.5).ywave(), 0.0);
    assert_feq!((-0.25).ywave(), -1.0);
    assert_feq!((0.0).ywave(), 0.0);
    assert_feq!((0.25).ywave(), 1.0);
    assert_feq!((0.5).ywave(), 0.0);

    // Test area further away
    assert_feq!((8.0).ywave(), 0.0);
    assert_feq!((8.25).ywave(), 1.0);
    assert_feq!((8.5).ywave(), 0.0);
    assert_feq!((8.75).ywave(), -1.0);
    assert_feq!((9.0).ywave(), 0.0);

    // Test area further into the negatives
    assert_feq!((-8.0).ywave(), 0.0);
    assert_feq!((-7.75).ywave(), 1.0);
    assert_feq!((-7.5).ywave(), 0.0);
    assert_feq!((-7.25).ywave(), -1.0);
    assert_feq!((-7.0).ywave(), 0.0);
}

struct Plasma {
    texture: Texture,
    pixel_data: Vec<u8>,
    time: f32
}

impl Plasma {
    fn new(renderer: &mut Renderer) -> Plasma {
        Plasma {
            texture: renderer.create_texture_streaming(PixelFormatEnum::RGB24, 640, 480).unwrap(),
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
        let mut value = 0.0;
        value += (x/23.0 + self.time).xwave();
        value += (x/13.0 + (y/17.0)*(self.time/2.0).xwave() ).xwave();
        let dx = (self.time/1.9).xwave()*200.0 + (WIDTH as f32)/2.0 - x;
        let dy = (self.time/3.1).ywave()*150.0 + (HEIGHT as f32)/2.0 - y;
        value += ((dx*dx + dy*dy).sqrt()/29.0 + self.time).xwave();
        return value;
    }

    fn calculate_color(&self, x: f32, y: f32) -> (u8, u8, u8) {
        let mut value = self.calculate_value(x, y);

        // scale value between 0 and 1
        value = value.fract().abs();
        value = if value < 0.5 { value*2.0 } else { (1.0 - value)*2.0 };

        let byte = (value * 255.0).round() as u8;
        (byte/4, byte/4 + 32, byte/2 + 64)
    }

    fn update(&mut self, renderer: &mut Renderer) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let (r, g, b) = self.calculate_color(x as f32, y as f32);
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
    let window = video.window("plasma", 640, 480).build().unwrap();

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
