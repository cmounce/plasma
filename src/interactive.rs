use genetics::{Genome, Population};
use gif::{Encoder, Frame};
use renderer::{PlasmaRenderer, Image};
use sdl2;
use sdl2::event::{Event, WindowEventId};
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Renderer, Texture};
use settings::PlasmaSettings;
use std::{f32, mem, thread};
use std::fs::File;
use std::time::{Duration, SystemTime};

pub fn run_interactive(settings: PlasmaSettings) {
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let width = settings.rendering.width;
    let height = settings.rendering.height;
    let window = video.window("plasma", width as u32, height as u32).resizable().build().unwrap();

    let mut renderer = window.renderer().build().unwrap();
    let mut plasma = Plasma::new(
        &mut renderer,
        width,
        height,
        settings.genetics.genome,
        settings.genetics.population
    );

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
                        Keycode::S => {
                            plasma.screenshot();
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
        let target_ms = 1000.0/settings.rendering.frames_per_second;
        let actual_ms = duration.subsec_nanos() as f32/1_000_000.0 +
            duration.as_secs() as f32*1000.0;
        if actual_ms > target_ms {
            println!("Target frame delay is {} but actual time taken is {}", target_ms, actual_ms);
        } else {
            let sleep_ms = (target_ms - actual_ms).max(0.0) as u64;
            thread::sleep(Duration::from_millis(sleep_ms));
        }
        plasma.add_time(target_ms.max(actual_ms)/1000.0);

        // Calculate time statistics
        if settings.output.verbose {
            avg_render_time += actual_ms;
            avg_render_time_count += 1;
            if avg_render_time_count >= 50 {
                println!("Average render time: {} ms", avg_render_time/(avg_render_time_count as f32));
                avg_render_time = 0.0;
                avg_render_time_count = 0;
            }
        }
    }
    if avg_render_time_count > 0 && settings.output.verbose {
        println!("Average render time: {} ms", avg_render_time/(avg_render_time_count as f32));
    }
}

pub struct Plasma {
    image: Image,
    renderer: PlasmaRenderer,
    population: Population,
    texture: Texture,
    time: f32
}

impl Plasma {
    pub fn new(renderer: &mut Renderer, width: usize, height: usize, starting_genome: Genome, population: Population) -> Plasma {
        Plasma {
            image: Image::new(width, height),
            population: population,
            renderer: PlasmaRenderer::new(starting_genome),
            texture: renderer.create_texture_streaming(PixelFormatEnum::RGB24, width as u32, height as u32).unwrap(),
            time: 0.0
        }
    }

    pub fn resize(&mut self, renderer: &mut Renderer, width: u32, height: u32) {
        self.image = Image::new(width as usize, height as usize);
        self.texture = renderer.create_texture_streaming(PixelFormatEnum::RGB24, width, height).unwrap();
    }

    pub fn update(&mut self, sdl_renderer: &mut Renderer) {
        self.renderer.render(&mut self.image, self.time/60.0);
        self.texture.update(None, &self.image.pixel_data[..], (self.image.width*3) as usize).unwrap();
        sdl_renderer.copy(&self.texture, None, None);
        sdl_renderer.present();
    }

    pub fn add_time(&mut self, time: f32) {
        self.time += time;
    }

    pub fn approve(&mut self) {
        let old_genome = self.replace_renderer();
        self.population.add(old_genome);
    }

    pub fn reject(&mut self) {
        self.replace_renderer();
    }

    pub fn export_current_genome(&self) {
        println!("{}", self.renderer.genome.to_base64());
    }

    pub fn screenshot(&self) {
        let frame = Frame::from_rgb(
            self.image.width as u16,
            self.image.height as u16,
            &self.image.pixel_data[..]
        );
        let mut file = File::create("screenshot.gif").unwrap();
        let mut encoder = Encoder::new(
            &mut file,
            self.image.width as u16,
            self.image.height as u16,
            &[]
        ).unwrap();
        encoder.write_frame(&frame).unwrap();
    }

    fn replace_renderer(&mut self) -> Genome {
        if let Some((g1, g2)) = self.population.get_pair() {
            let child = g1.breed(g2);
            let new_renderer = PlasmaRenderer::new(child);
            let old_renderer = mem::replace(&mut self.renderer, new_renderer);
            old_renderer.genome
        } else {
            panic!("Could not get a breeding pair from population struct");
        }
    }
}
