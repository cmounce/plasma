use colormapper::{NUM_COLOR_GENES,CONTROL_POINT_GENE_SIZE};
use formulas::{NUM_FORMULA_GENES,FORMULA_GENE_SIZE};
use genetics::{Chromosome,Genome,Population};
use renderer::{PlasmaRenderer, Image};
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Renderer;
use sdl2::render::Texture;
use std::{f32,mem};

const STARTING_POPULATION_SIZE: usize = 8;
const MAX_POPULATION_SIZE: usize = 32;

pub struct Plasma {
    image: Image,
    renderer: PlasmaRenderer,
    population: Population,
    texture: Texture,
    time: f32
}

impl Plasma {
    pub fn new(renderer: &mut Renderer, width: u32, height: u32) -> Plasma {
        fn rand_genome() -> Genome {
            Genome {
                pattern: Chromosome::rand(NUM_FORMULA_GENES, FORMULA_GENE_SIZE),
                color: Chromosome::rand(NUM_COLOR_GENES, CONTROL_POINT_GENE_SIZE)
            }
        }
        let mut population = Population::new(MAX_POPULATION_SIZE);
        for _ in 0..STARTING_POPULATION_SIZE {
            population.add(rand_genome());
        }
        Plasma {
            image: Image::new(width as usize, height as usize),
            population: population,
            renderer: PlasmaRenderer::new(rand_genome()),
            texture: renderer.create_texture_streaming(PixelFormatEnum::RGB24, width, height).unwrap(),
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
