use genetics::Genome;
use gif::{Encoder, Frame};
use renderer::{Image, PlasmaRenderer};
use std::fs::File;

pub struct NonInteractiveParams {
    pub filepath: String,
    pub genome: Genome,
    pub width: u32,
    pub height: u32,
    pub fps: u8
}

pub fn output_gif(params: NonInteractiveParams) {
    let mut renderer = PlasmaRenderer::new(params.genome);
    let mut image = Image::new(params.width as usize, params.height as usize);

    let mut file = File::create(params.filepath).unwrap();
    let mut encoder = Encoder::new(
        &mut file,
        image.width as u16,
        image.height as u16,
        &[]
    ).unwrap();

    for seconds in 0..60 {
        for frames in 0..params.fps {
            let time = seconds as f32 + frames as f32 / params.fps as f32;
            let adj_time = time/60.0;
            renderer.render(&mut image, adj_time);
            let frame = Frame::from_rgb(
                image.width as u16,
                image.height as u16,
                &image.pixel_data[..]
            );
            encoder.write_frame(&frame).unwrap();
        }
    }
}
