use gif::{Encoder, Frame, SetParameter, Repeat};
use gradient::Color;
use renderer::{Image, PlasmaRenderer};
use settings::{OutputMode,PlasmaSettings};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs::File;
use std::iter;
use std::ops::Range;

pub fn output_gif(settings: PlasmaSettings) {
    let mut renderer = PlasmaRenderer::new(&settings.genetics.genome, &settings.rendering);
    let mut image_rgb = Image::new(settings.rendering.width, settings.rendering.height);

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
        image_rgb.width as u16,
        image_rgb.height as u16,
        &palette_bytes[..]
    ).unwrap();
    encoder.set(Repeat::Infinite).unwrap();

    let mut previous_pixels: Option<Vec<u8>> = None;
    let fps = settings.rendering.frames_per_second;
    for seconds in 0..60 {
        for frames in 0..(fps as u8) {
            let time = seconds as f32 + frames as f32 / fps;
            let adj_time = time/60.0;
            renderer.render(&mut image_rgb, adj_time);

            // Convert RGB image to indexed
            let mut pixels: Vec<u8> = image_rgb.pixel_data.chunks(3).map(|slice| {
                let rgb = (slice[0], slice[1], slice[2]);
                *palette_map.get(&rgb).expect("Image contained color not in palette")
            }).collect();

            // Optimize image by making pixels transparent
            if let Some(previous_pixels) = previous_pixels {
                optimize_pixels(&previous_pixels[..], &mut pixels[..]);
            }

            // Create frame from image
            {
                let mut frame = Frame::default();
                frame.width = image_rgb.width as u16;
                frame.height = image_rgb.height as u16;
                frame.delay = (100.0/settings.rendering.frames_per_second).round() as u16;
                frame.buffer = Cow::Borrowed(&pixels);
                frame.transparent = Some(0);
                encoder.write_frame(&frame).unwrap();
            }
            previous_pixels = Some(pixels);
        }
    }
}

fn optimize_pixels(previous_pixels: &[u8], pixels: &mut [u8]) {
    // Find runs of pixels that didn't change from one frame to the next.
    // These runs are candidates to be made transparent.
    let runs;
    {
        let unchanged_pixels = previous_pixels.iter().zip(pixels.iter()).map(|(a, b)| a == b);
        let single_pixel_runs = unchanged_pixels.enumerate().filter_map(|(i, unchanged)| {
            if unchanged {
                Some(i..(i+1))
            } else {
                None
            }
        });
        runs = single_pixel_runs.fold(vec![], |mut runs, run| {
            if runs.is_empty() {
                runs.push(run);
            } else {
                let last_index = runs.len() - 1;
                if runs[last_index].end == run.start {
                    runs[last_index].end = run.end;
                } else {
                    runs.push(run);
                }
            }
            runs
        });
    }

    // Eliminate runs which would increase the encoded size of a RLE pixel stream if they were
    // made transparent. The GIF will be encoded using LZW, not RLE, but RLE is a good heuristic.
    let good_runs: Vec<Range<usize>> = runs.into_iter().filter(|run| {
        let mut cost = 0;
        if run.start > 0 && pixels[run.start - 1] == pixels[run.start] {
            cost += 1;
        }
        if run.end < pixels.len() && pixels[run.end - 1] == pixels[run.end] {
            cost += 1;
        }
        let benefit = pixels[run.clone()].windows(2).filter(|w| w[0] != w[1]).take(2).count();
        cost <= benefit
    }).collect();

    // Actually make pixels transparent
    for run in good_runs {
        for i in run {
            pixels[i] = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::optimize_pixels;

    fn assert_optimize(previous_pixels: &[u8], pixels: &mut [u8], expected_optimization: &[u8]) {
        optimize_pixels(previous_pixels, pixels);
        assert_eq!(pixels, expected_optimization);
    }

    #[test]
    fn test_optimize_pixels() {
        // Positive benefit
        assert_optimize(&[1,2,3,4,5,6], &mut [7,2,3,4,5,8], &[7,0,0,0,0,8]);
        assert_optimize(&[1,2,3,4,5,6], &mut [1,2,7,8,5,6], &[0,0,7,8,0,0]);
        assert_optimize(&[1,1,2,2,3,3], &mut [1,1,2,2,4,4], &[0,0,0,0,4,4]);
        assert_optimize(&[1,1,2,2,3,3], &mut [4,4,2,2,3,3], &[4,4,0,0,0,0]);

        // Neutral
        assert_optimize(&[1,1,1,1,1,1], &mut [2,1,1,1,1,2], &[2,0,0,0,0,2]);
        assert_optimize(&[1,1,2,2,3,3], &mut [1,4,4,4,3,3], &[0,4,4,4,0,0]);
        assert_optimize(&[1,1,2,2,3,3], &mut [1,1,4,4,4,3], &[0,0,4,4,4,0]);

        // Negative benefit
        assert_optimize(&[1,1,2,2,1,1], &mut [2,2,2,2,2,2], &[2,2,2,2,2,2]);
        assert_optimize(&[1,1,2,2,2,2], &mut [2,2,2,2,2,2], &[2,2,2,2,2,2]);
        assert_optimize(&[2,2,2,2,1,1], &mut [2,2,2,2,2,2], &[2,2,2,2,2,2]);
    }
}
