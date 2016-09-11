use colormapper::{ColorMapper,CONTROL_POINT_GENE_SIZE};
use fastmath::FastMath;
use genetics::{Chromosome,Genome,Population};
use gradient::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Renderer;
use sdl2::render::Texture;
use std::{f32,mem};

const POPULATION_SIZE: usize = 8;

struct Image {
    width: usize,
    height: usize,
    pixel_data: Vec<u8>
}

pub struct Plasma {
    image: Image,
    renderer: PlasmaRenderer,
    population: Population,
    texture: Texture,
    time: f32
}

struct PlasmaRenderer {
    genome: Genome,
    color_mapper: ColorMapper
}

impl Image{
    fn new(width: usize, height: usize) -> Image {
        Image {
            width: width,
            height: height,
            pixel_data: vec![0; width*height*3]
        }
    }

    fn plot(&mut self, x: usize, y: usize, color: Color) {
        let offset = (x + y*self.width)*3;
        self.pixel_data[offset] = color.r;
        self.pixel_data[offset + 1] = color.g;
        self.pixel_data[offset + 2] = color.b;
    }
}

impl Plasma {
    pub fn new(renderer: &mut Renderer, width: u32, height: u32) -> Plasma {
        fn rand_genome() -> Genome {
            Genome {
                pattern: Chromosome::rand(1, 1),
                color: Chromosome::rand(10, CONTROL_POINT_GENE_SIZE)
            }
        }
        let mut population = Population::new(POPULATION_SIZE);
        for _ in 0..POPULATION_SIZE {
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

    pub fn update(&mut self, sdl_renderer: &mut Renderer) {
        self.renderer.render(&mut self.image, self.time);
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

impl PlasmaRenderer {
    fn new(genome: Genome) -> PlasmaRenderer {
        let color_mapper = ColorMapper::new(&genome.color);
        PlasmaRenderer {
            genome: genome,
            color_mapper: color_mapper
        }
    }

    fn render(&self, image: &mut Image, time: f32) {
        let scale = 1.0/((image.width as f32).min(image.height as f32));
        let adj_time = time; // TODO: convert to use time.wrap();
        for y in 0..image.height {
            for x in 0..image.width {
                let color = self.calculate_color(x as f32 * scale, y as f32 * scale, adj_time);
                image.plot(x, y, color);
            }
        }
    }

    fn calculate_color(&self, x: f32, y: f32, time: f32) -> Color {
        let x_adj = x*200.0;
        let y_adj = y*200.0;

        let mut value = 0.0;
        value += ((x_adj/23.0 + time)/10.0).wave();
        value += ((x_adj/13.0 + (y_adj/17.0)*(time/20.0).wave() )/10.0).wave();
        let dx = (time/19.0).wave()*75.0 + 100.0 - x_adj;
        let dy = (time/31.0 + 0.5).wave()*75.0 + 100.0 - y_adj;
        value += ((dx*dx + dy*dy).sqrt()/290.0 + time/10.0).wave();

        self.color_mapper.convert(value)
    }
}

#[cfg(test)]
mod tests {

}
