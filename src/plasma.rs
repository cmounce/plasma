use colormapper::ColorMapper;
use fastmath::FastMath;
use genetics::Gene;
use genetics::Genome;
use gradient::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Renderer;
use sdl2::render::Texture;
use std::f32;

// TODO: Add methods for approving/rejecting the current pattern/color
pub struct Plasma {
    renderer: PlasmaRenderer,
    texture: Texture,
    pixel_data: Vec<u8>,
    width: u32,
    height: u32,
    time: f32
}

struct PlasmaRenderer {
    color_mapper: ColorMapper
}

impl Plasma {
    pub fn new(renderer: &mut Renderer, width: u32, height: u32) -> Plasma {
        Plasma {
            renderer: PlasmaRenderer::new(),
            texture: renderer.create_texture_streaming(PixelFormatEnum::RGB24, width, height).unwrap(),
            pixel_data: vec![0; (width*height*3) as usize],
            time: 0.0,
            width: width,
            height: height
        }
    }

    pub fn update(&mut self, renderer: &mut Renderer) {
        self.renderer.render(&mut self.pixel_data[..], self.width as usize, self.height as usize, self.time);
        self.texture.update(None, &self.pixel_data[..], (self.width*3) as usize).unwrap();
        renderer.copy(&self.texture, None, None);
        renderer.present();
    }

    pub fn add_time(&mut self, time: f32) {
        self.time += time;
    }
}

impl PlasmaRenderer {
    fn new() -> PlasmaRenderer {
        PlasmaRenderer {
            color_mapper: ColorMapper::new()
        }
    }

    fn render(&self, pixel_data: &mut [u8], width: usize, height: usize, time: f32) {
        // TODO: Add some kind of image type that has pixel data, width, height
        let scale = 1.0/((width as f32).min(height as f32));
        let adj_time = time; // TODO: convert to use time.wrap();
        for y in 0..height {
            for x in 0..width {
                let color = self.calculate_color(x as f32 * scale, y as f32 * scale, adj_time);
                self.plot(pixel_data, width, height, x, y, color);
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

    fn plot(&self, pixel_data: &mut [u8], width: usize, height: usize, x: usize, y: usize, color: Color) {
        let offset = (x + y*width)*3;
        pixel_data[offset] = color.r;
        pixel_data[offset + 1] = color.g;
        pixel_data[offset + 2] = color.b;
    }
}

#[cfg(test)]
mod tests {

}
