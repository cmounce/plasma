use fastmath::FastMath;
use colormapper::ColorMapper;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Renderer;
use sdl2::render::Texture;
use std::f32;

// TODO: Pass these as parameters instead of as constants
const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

// TODO: Add methods for approving/rejecting the current pattern/color
pub struct Plasma {
    color_mapper: ColorMapper,
    texture: Texture,
    pixel_data: Vec<u8>,
    time: f32
}

impl Plasma {
    pub fn new(renderer: &mut Renderer) -> Plasma {
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

    pub fn update(&mut self, renderer: &mut Renderer) {
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

    pub fn add_time(&mut self, time: f32) {
        self.time += time;
    }
}
