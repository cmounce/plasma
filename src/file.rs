use gif::{Encoder, Frame, SetParameter, Repeat};
use renderer::{Image, PlasmaRenderer};
use settings::{OutputMode,PlasmaSettings};
use std::fs::File;

pub fn output_gif(settings: PlasmaSettings) {
    let mut renderer = PlasmaRenderer::new(&settings.genetics.genome, &settings.rendering);
    let mut image = Image::new(settings.rendering.width, settings.rendering.height);

    let path = match settings.output.mode {
        OutputMode::File{path} => path,
        _ => panic!("OutputMode must be File")
    };
    let mut file = File::create(path).unwrap();
    let mut encoder = Encoder::new(
        &mut file,
        image.width as u16,
        image.height as u16,
        &[]
    ).unwrap();
    encoder.set(Repeat::Infinite).unwrap();

    let fps = settings.rendering.frames_per_second;
    for seconds in 0..60 {
        for frames in 0..(fps as u8) {
            let time = seconds as f32 + frames as f32 / fps;
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
