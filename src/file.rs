use gif::{Encoder, Frame, SetParameter, Repeat};
use renderer::{Image, PlasmaRenderer};
use settings::{OutputMode,PlasmaSettings};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs::File;

pub fn output_gif(settings: PlasmaSettings) {
    let mut renderer = PlasmaRenderer::new(&settings.genetics.genome, &settings.rendering);
    let mut image = Image::new(settings.rendering.width, settings.rendering.height);

    // Set up global palette
    let mut palette_map = BTreeMap::new();
    let mut palette_bytes = vec![];
    for (index, color) in renderer.get_palette().iter().enumerate() {
        palette_map.insert((color.r, color.g, color.b), index as u8);
        palette_bytes.push(color.r);
        palette_bytes.push(color.g);
        palette_bytes.push(color.b);
    }

    let path = match settings.output.mode {
        OutputMode::File{path} => path,
        _ => panic!("OutputMode must be File")
    };
    let mut file = File::create(path).unwrap();
    let mut encoder = Encoder::new(
        &mut file,
        image.width as u16,
        image.height as u16,
        &palette_bytes[..]
    ).unwrap();
    encoder.set(Repeat::Infinite).unwrap();

    let fps = settings.rendering.frames_per_second;
    for seconds in 0..60 {
        for frames in 0..(fps as u8) {
            let time = seconds as f32 + frames as f32 / fps;
            let adj_time = time/60.0;
            renderer.render(&mut image, adj_time);

            // Create frame from image
            let indexed_image: Vec<u8> = image.pixel_data.chunks(3).map(|slice| {
                let rgb = (slice[0], slice[1], slice[2]);
                *palette_map.get(&rgb).expect("Image contained color not in palette")
            }).collect();
            let mut frame = Frame::default();
            frame.width = image.width as u16;
            frame.height = image.height as u16;
            frame.delay = (100.0/settings.rendering.frames_per_second).round() as u16;
            frame.buffer = Cow::Borrowed(&indexed_image);

            encoder.write_frame(&frame).unwrap();
        }
    }
}
