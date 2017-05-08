use gif::{Encoder, Frame, SetParameter, Repeat};
use gradient::Color;
use renderer::{Image, PlasmaRenderer};
use settings::{OutputMode, PlasmaSettings};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::ops::Range;

pub fn output_gif(settings: PlasmaSettings) {
    // Render all the frames at once
    let mut renderer = PlasmaRenderer::new(&settings.genetics.genome, &settings.rendering);
    let num_frames = (settings.rendering.frames_per_second*settings.rendering.loop_duration).
        round() as usize;
    let times = (0..num_frames).map(|i| i as f32/num_frames as f32);
    let frames: Vec<Image> = times.map(|time| {
        let mut image = Image::new(settings.rendering.width, settings.rendering.height);
        renderer.render(&mut image, time);
        image
    }).collect();

    // Convert frames to indexed
    let mut palette = renderer.get_palette();
    let mut indexed_frames: Vec<Vec<u8>> = {
        let mut palette_map = BTreeMap::new();
        for (index, color) in palette.iter().enumerate() {
            palette_map.insert((color.r, color.g, color.b), index as u8);
        }
        frames.iter().map(|frame|
            frame.pixel_data.chunks(3).map(|slice| {
                let rgb = (slice[0], slice[1], slice[2]);
                *palette_map.get(&rgb).expect("Image contained color not in palette")
            }).collect()
        ).collect()
    };

    // Encode a GIF as-is (no transparent pixels)
    let mut gif_bytes = encode_gif(&indexed_frames[..], &palette[..], &settings, false);

    // Encode the GIF again, but this time try to optimize it by using transparent pixels
    if palette.len() < 256 {
        // Add transparency to the frames
        palette.insert(0, Color::new(0, 0, 0)); // Add transparent palette entry
        for indexed_frame in indexed_frames.iter_mut() {
            for index in indexed_frame.iter_mut() {
                *index += 1; // Adjust existing indexes to accommodate transparency
            }
        }

        // Optimize pixels
        let mut previous_indexed_frame = indexed_frames[0].clone();
        for i in 1..indexed_frames.len() {
            let original_indexed_frame = indexed_frames[i].clone();
            optimize_pixels(&previous_indexed_frame[..], &mut indexed_frames[i][..]);
            previous_indexed_frame = original_indexed_frame;
        }

        let new_gif_bytes = encode_gif(&indexed_frames[..], &palette[..], &settings, true);
        if new_gif_bytes.len() < gif_bytes.len() {
            // Only use transparency if it results in a smaller file
            gif_bytes = new_gif_bytes;
        }
    }

    // Actually output the gif
    let path = match settings.output.mode {
        OutputMode::File{path} => path,
        _ => panic!("OutputMode must be File")
    };
    let mut file = File::create(path).expect("Couldn't open file");
    file.write_all(&gif_bytes[..]).expect("Couldn't write GIF data to file");
}

fn encode_gif(indexed_frames: &[Vec<u8>], palette: &[Color],
              settings: &PlasmaSettings, transparent_index_zero: bool) -> Vec<u8> {
    // Calculate frame delay
    let frame_delay_seconds = settings.rendering.loop_duration/(indexed_frames.len() as f32);
    let frame_delay_centiseconds = (frame_delay_seconds*100.0).round() as u16;

    // Output GIF byte stream
    let mut output = vec![];
    {
        let palette_bytes: Vec<u8> = palette.iter().flat_map(|c| vec![c.r, c.g, c.b]).collect();
        let mut encoder = Encoder::new(
            &mut output,
            settings.rendering.width as u16,
            settings.rendering.height as u16,
            &palette_bytes[..]
        ).unwrap();
        encoder.set(Repeat::Infinite).unwrap();

        for indexed_frame in indexed_frames.iter() {
            let mut frame = Frame::default();
            frame.width = settings.rendering.width as u16;
            frame.height = settings.rendering.height as u16;
            frame.delay = frame_delay_centiseconds;
            frame.buffer = Cow::Borrowed(indexed_frame);
            if transparent_index_zero {
                frame.transparent = Some(0);
            }
            encoder.write_frame(&frame).unwrap();
        }
    }
    output
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
