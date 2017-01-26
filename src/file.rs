use gif::{Encoder, Frame, SetParameter, Repeat};
use gradient::Color;
use renderer::{Image, PlasmaRenderer};
use settings::{OutputMode,PlasmaSettings};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs::File;
use std::iter;

pub fn output_gif(settings: PlasmaSettings) {
    let mut renderer = PlasmaRenderer::new(&settings.genetics.genome, &settings.rendering);
    let mut image = Image::new(settings.rendering.width, settings.rendering.height);

    // Set up global palette
    let image_colors = renderer.get_palette();
    let transparency_color = Color::new(0, 0, 0);
    let colors = iter::once(&transparency_color).chain(image_colors.iter());
    let mut palette_map = BTreeMap::new();
    let mut palette_bytes = vec![];
    for (index, color) in colors.enumerate() {
        palette_map.insert((color.r, color.g, color.b), index as u8);
        palette_bytes.push(color.r);
        palette_bytes.push(color.g);
        palette_bytes.push(color.b);
    }

    // Open file, initialize GIF encoder
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

    let mut previous_frame: Option<Frame> = None;
    let fps = settings.rendering.frames_per_second;
    for seconds in 0..60 {
        for frames in 0..(fps as u8) {
            let time = seconds as f32 + frames as f32 / fps;
            let adj_time = time/60.0;
            renderer.render(&mut image, adj_time);

            // Convert RGB image to indexed
            let mut indexed_image: Vec<u8> = image.pixel_data.chunks(3).map(|slice| {
                let rgb = (slice[0], slice[1], slice[2]);
                *palette_map.get(&rgb).expect("Image contained color not in palette")
            }).collect();
            if let Some(previous_frame) = previous_frame {
                // Optimize image by making pixels transparent
                for (i, data) in previous_frame.buffer.iter().enumerate() {
                    if indexed_image[i] == *data {
                        indexed_image[i] = 0;
                    }
                }
            }

            // Create frame from image
            let mut frame = Frame::default();
            frame.width = image.width as u16;
            frame.height = image.height as u16;
            frame.delay = (100.0/settings.rendering.frames_per_second).round() as u16;
            frame.buffer = Cow::Owned(indexed_image);
            frame.transparent = Some(0);

            encoder.write_frame(&frame).unwrap();
            previous_frame = Some(frame);
        }
    }
}
