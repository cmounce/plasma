use asyncrenderer::AsyncRenderer;
use color::colormapper::{NUM_COLOR_GENES, CONTROL_POINT_GENE_SIZE};
use fastmath::FastMath;
use formulas::{NUM_FORMULA_GENES, FORMULA_GENE_SIZE};
use genetics::{Chromosome, Genome, Population};
use sdl2;
use sdl2::event::{Event, WindowEventId};
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Texture;
use settings::PlasmaSettings;
use std::f32;
use std::time::Instant;

struct PlasmaState {
    clock_instant: Instant,
    current_texture: Texture,
    current_genome: Genome,
    frame_deadline_seconds: f64,
    population: Population,
    renderer: AsyncRenderer,
    width: u32,
    height: u32
}

impl PlasmaState {
    fn approve_current_genome(&mut self) {
        let old_genome = self.current_genome.clone();
        self.population.add(old_genome);
        let new_genome = self.population.breed();
        self.set_genome(new_genome);
    }

    fn reject_current_genome(&mut self) {
        let genome = self.population.breed();
        self.set_genome(genome);
    }

    fn randomize_current_genome(&mut self) {
        self.set_genome(Genome {
            pattern: Chromosome::rand(NUM_FORMULA_GENES, FORMULA_GENE_SIZE),
            color: Chromosome::rand(NUM_COLOR_GENES, CONTROL_POINT_GENE_SIZE)
        });
    }

    fn set_genome(&mut self, genome: Genome) {
        self.current_genome = genome;
        self.clock_instant = Instant::now(); // Reset the clock
        self.renderer.set_genome(&self.current_genome);
        self.renderer.render(self.width as usize, self.height as usize, 0.0);
        self.frame_deadline_seconds = 0.0;
    }

    fn clock_seconds(&self) -> f64 {
        let duration = self.clock_instant.elapsed();
        duration.as_secs() as f64 + (duration.subsec_nanos() as f64/1_000_000_000.0)
    }
}

pub fn run_interactive(settings: PlasmaSettings) {
    // Initialize SDL structs
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let window = video.window(
        "plasma",
        settings.rendering.width as u32,
        settings.rendering.height as u32
    ).resizable().build().unwrap();
    let mut sdl_renderer = window.renderer().build().unwrap();
    let mut event_pump = sdl.event_pump().unwrap();

    // Init screen to black via an initial 1x1 texture
    let mut texture = sdl_renderer.create_texture_streaming(PixelFormatEnum::RGB24, 1, 1).unwrap();
    texture.update(None, &[0, 0, 0], 3).unwrap();
    sdl_renderer.copy(&texture, None, None);
    sdl_renderer.present();

    // Initialize plasma state
    let mut state = PlasmaState {
        clock_instant: Instant::now(),
        current_texture: texture,
        current_genome: settings.genetics.genome,
        frame_deadline_seconds: 0.0,
        population: settings.genetics.population,
        renderer: AsyncRenderer::new(&settings.rendering),
        width: settings.rendering.width as u32,
        height: settings.rendering.height as u32
    };

    // Start an async render on the current_genome
    state.renderer.set_genome(&state.current_genome);
    state.renderer.render(state.width as usize, state.height as usize, 0.0);

    // Calculate some useful constants
    let frame_delay_seconds = 1.0/(settings.rendering.frames_per_second as f64);
    let time_scale_factor = 1.0/settings.rendering.loop_duration as f64;

    loop {
        // If a frame is due, put it on the screen
        if state.frame_deadline_seconds <= state.clock_seconds() {
            if let Some(image) = state.renderer.get_image() {
                // We have a frame, and it's due. Display it!
                // But before we do, start a render of the next frame
                state.frame_deadline_seconds = state.clock_seconds() + frame_delay_seconds;
                let adj_time = ((state.frame_deadline_seconds*time_scale_factor) as f32).wrap();
                state.renderer.render(state.width as usize, state.height as usize, adj_time);

                // Resize texture if necessary
                let query = state.current_texture.query();
                if (image.width, image.height) != (query.width as usize, query.height as usize) {
                    state.current_texture = sdl_renderer.
                        create_texture_streaming(PixelFormatEnum::RGB24, state.width, state.height).unwrap();
                }
                // Update texture, screen
                state.current_texture.update(None, &image.pixel_data[..], image.width*3).unwrap();
                sdl_renderer.copy(&state.current_texture, None, None);
                sdl_renderer.present();
            }
        }

        // Calculate wait_time
        let wait_time_seconds = frame_delay_seconds.min(0.005);

        // Wait up to wait_time for events
        let wait_time_ms = (wait_time_seconds*1000.0).round() as u32;
        let event_vec = if let Some(event) = event_pump.wait_event_timeout(wait_time_ms) {
            vec![event]
        } else {
            vec![]
        };
        let events = event_vec.into_iter().chain(event_pump.poll_iter());

        // Process events
        for event in events {
            match event {
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    match keycode {
                        // User approves current genome
                        Keycode::Equals | Keycode::Plus | Keycode::KpPlus => {
                            state.approve_current_genome();
                        }
                        // User rejects current genome
                        Keycode::Minus | Keycode::Underscore | Keycode::KpMinus => {
                            state.reject_current_genome();
                        }
                        // Export current genome
                        Keycode::P => {
                            println!("{}", state.current_genome.to_base64());
                        }
                        Keycode::R => {
                            state.randomize_current_genome();
                        }
                        _ => ()
                    }
                }
                Event::Window {
                    win_event_id: WindowEventId::Resized,
                    data1: new_width,
                    data2: new_height,
                    ..
                } => {
                    state.width = new_width as u32;
                    state.height = new_height as u32;
                    sdl_renderer.copy(&state.current_texture, None, None);
                    sdl_renderer.present();
                }
                Event::Quit { .. } => return,
                _ => ()
            }
        }
        // End of event processing

        // End of main loop
    }
}
