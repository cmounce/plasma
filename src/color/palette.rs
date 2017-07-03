use cgmath::Vector3;
use cgmath::prelude::*;
use ordered_float::OrderedFloat;
use std::{cmp, u16};
use std::collections::HashSet;
use std::ops::Index;
use super::LinearColor;

const BAYER_MATRIX: [[u8; 8]; 8] = [
    [ 0, 48, 12, 60,  3, 51, 15, 63],
    [32, 16, 44, 28, 35, 19, 47, 31],
    [ 8, 56,  4, 52, 11, 59,  7, 55],
    [40, 24, 36, 20, 43, 27, 39, 23],
    [ 2, 50, 14, 62,  1, 49, 13, 61],
    [34, 18, 46, 30, 33, 17, 45, 29],
    [10, 58,  6, 54,  9, 57,  5, 53],
    [42, 26, 38, 22, 41, 25, 37, 21]
];

pub struct Palette {
    pub colors: Vec<LinearColor>
}

// A dithering pattern that approximates a specific color
#[derive(Clone, Copy)]
pub struct DitherPattern {
    palette_indexes: [u16; 4],      // When dithering, mix these colors (up to 4)
    palette_proportions: [u8; 4]    // in these proportions (total of 64)
}

// Private helpers for working with LinearColors
impl LinearColor {
    fn squared_distance(&self, other: LinearColor) -> u64 {
        fn partial(x: u16, y: u16) -> u64 {
            let delta = (x as i64) - (y as i64);
            (delta*delta) as u64
        }
        partial(self.r, other.r) + partial(self.g, other.g) + partial(self.b, other.b)
    }

    fn average(colors: &[LinearColor]) -> LinearColor {
        let mut totals = [0.0, 0.0, 0.0];
        for color in colors {
            totals[0] += color.r as f32;
            totals[1] += color.g as f32;
            totals[2] += color.b as f32;
        }
        let num_colors = colors.len() as f32;
        let avg_component = |i| f32::round(totals[i]/num_colors) as u16;
        LinearColor {
            r: avg_component(0),
            g: avg_component(1),
            b: avg_component(2)
        }
    }
}

impl Palette {
    // Generate an optimized palette based on the provided color samples
    pub fn new(palette_size: usize, samples: &[LinearColor], maximize_range: bool) -> Palette {
        assert!(palette_size >= 2);
        assert!(palette_size <= u16::MAX as usize);

        // Shortcut: if we're not reducing the number of colors, just use samples as our colors
        if samples.len() <= palette_size {
            let mut colors = Vec::with_capacity(palette_size);
            colors.extend_from_slice(samples);
            while colors.len() < palette_size {
                colors.push(LinearColor::new(0, 0, 0));
            }
            return Palette { colors: colors };
        }

        // Create an initial palette by subsampling the provided samples
        let mut palette = Palette {
            colors: Vec::with_capacity(palette_size)
        };
        let subsample_distance = samples.len() as f32/palette_size as f32;
        for i in 0..palette_size {
            let subsample_index = (i as f32 * subsample_distance) as usize;
            palette.colors.push(samples[subsample_index]);
        }

        // Pin the outermost palette entries to the edges of the color space
        let pinned_palette_indexes: HashSet<usize> = if maximize_range {
            // Calculate repelling forces among palette entries
            let palette_vectors: Vec<_> = palette.colors.iter().map(|c| c.to_vec3()).collect();
            let repelling_forces: Vec<Vector3<f32>> = palette_vectors.iter().map(|color| {
                let raw_deltas = palette_vectors.iter().map(|other_color| color - other_color);
                let scaled_forces = raw_deltas.map(|raw| {
                    let mag2 = raw.magnitude2();
                    let scale = mag2*mag2; // Repelling force = 1/(distance**3)
                    if scale > 0.0 {
                        raw/scale
                    } else {
                        raw
                    }
                });
                scaled_forces.sum()
            }).collect();

            // Figure out which palette indexes are on the outside of the color space
            let outside_palette_indexes: HashSet<_> = repelling_forces.iter().enumerate().filter_map(|(i, force)| {
                let color = palette_vectors[i];
                if palette_vectors.iter().any(|other_color| force.dot(other_color - color) > 0.0) {
                    None
                } else {
                    Some(i)
                }
            }).collect();

            // Update the outside palette entries to be the most extreme sample
            for &palette_index in outside_palette_indexes.iter() {
                let force = repelling_forces[palette_index];
                palette.colors[palette_index] = *samples.iter().max_by_key(|sample| {
                    OrderedFloat(sample.to_vec3().dot(force))
                }).unwrap();
            }

            outside_palette_indexes
        } else {
            HashSet::new()
        };

        // Optimize the palette by doing k-means clustering on the samples.
        // Each of the k means will become a color in the optimized palette.
        let mut palette_updated = true;
        while palette_updated {
            // Group samples by each one's closest palette color
            let mut palette_index_to_samples = vec![vec![]; palette_size];
            for &sample in samples {
                let palette_index = palette.get_nearest_index(sample);
                palette_index_to_samples[palette_index].push(sample);
            }

            // Replace each palette color with the average of its corresponding sample group
            palette_updated = false;
            for (palette_index, nearest_samples) in palette_index_to_samples.iter().enumerate() {
                if nearest_samples.len() > 0 && !pinned_palette_indexes.contains(&palette_index) {
                    let average = LinearColor::average(nearest_samples);
                    if palette.colors[palette_index] != average {
                        palette.colors[palette_index] = average;
                        palette_updated = true;
                    }
                }
            }
        }
        palette
    }

    // Given an arbitrary color, returns the index of the nearest palette color
    pub fn get_nearest_index(&self, color: LinearColor) -> usize {
        let index_color = self.colors.iter().enumerate().min_by_key(|&(_, palette_color)|
            color.squared_distance(*palette_color)
        );
        index_color.expect("Palette has no colors").0
    }

    // Given an arbitrary color, returns a DitherPattern that approximates that color
    pub fn get_dither_pattern(&self, color: LinearColor) -> DitherPattern {
        // Figure out which colors should be mixed together to make the target color.
        // This is based off of Yliluoma's work: http://bisqwit.iki.fi/story/howto/dither/jy/
        let max_colors = 4;
        let max_new_color_iters = 16;
        let mut subpalette = Palette {
            colors: Vec::with_capacity(max_colors)
        };
        let mut palette_indexes = Vec::with_capacity(max_colors);
        let mut counts = Vec::with_capacity(max_colors);
        let mut errors: [i32; 3] = [0, 0, 0];
        for i in 0..64 {
            // Calculate target color = (original color - accumulated error)
            let mut target = color;
            let sub_error = |component, error| {
                // We can't use saturating_sub() here, because we're mixing i32 and u16
                cmp::min(cmp::max(0, component as i32 - error), u16::MAX as i32) as u16
            };
            target.r = sub_error(target.r, errors[0]);
            target.g = sub_error(target.g, errors[1]);
            target.b = sub_error(target.b, errors[2]);

            // Find the nearest color to the target color
            let allow_new_colors = i < max_new_color_iters && subpalette.colors.len() < max_colors;
            let (nearest_palette_index, nearest_subpalette_index) = if allow_new_colors {
                // Search the whole palette
                let palette_index = self.get_nearest_index(target);
                let subpalette_index = palette_indexes.iter().position(|x| *x == palette_index);
                (palette_index, subpalette_index)
            } else {
                // Search just the subpalette
                let subpalette_index = subpalette.get_nearest_index(target);
                (palette_indexes[subpalette_index], Some(subpalette_index))
            };

            // Process the color we found
            if let Some(subpalette_index) = nearest_subpalette_index {
                // We've already seen this color, so just increment the count
                counts[subpalette_index] += 1;
            } else {
                // We've found a new color, so add it to our data structures
                subpalette.colors.push(self[nearest_palette_index]);
                palette_indexes.push(nearest_palette_index);
                counts.push(1);
            }

            // Update our accumulated error
            let last_color = self[nearest_palette_index];
            errors[0] += last_color.r as i32 - color.r as i32;
            errors[1] += last_color.g as i32 - color.g as i32;
            errors[2] += last_color.b as i32 - color.b as i32;
        }

        // Assemble data into a DitherPattern struct.
        let mut retval = DitherPattern {
            palette_indexes: [0, 0, 0, 0],
            palette_proportions: [0, 0, 0, 0]
        };
        let mut indexes_counts: Vec<_> = palette_indexes.iter().zip(counts.iter()).collect();
        /*
         * Sorting the colors improves the consistency of dithered output.
         * Imagine dithering a black->white gradient with a palette of black and white: if we
         * didn't sort the colors, black and white would switch places at the halfway point,
         * which would create a visible seam in the dithered pattern.
         */
        indexes_counts.sort();
        for (i, &(&palette_index, &count)) in indexes_counts.iter().enumerate() {
            retval.palette_indexes[i] = palette_index as u16;
            retval.palette_proportions[i] = count as u8;
        }
        retval
    }
}

impl Index<usize> for Palette {
    type Output = LinearColor;

    fn index(&self, index: usize) -> &LinearColor {
        &self.colors[index]
    }
}

impl DitherPattern {
    pub fn get_palette_index(&self, x: usize, y: usize) -> usize {
        let bayer_value = BAYER_MATRIX[y % 8][x % 8];
        let mut cumulative_proportion = self.palette_proportions[0];
        let mut dither_index = 0;
        while cumulative_proportion <= bayer_value {
            dither_index += 1;
            cumulative_proportion += self.palette_proportions[dither_index];
        }
        self.palette_indexes[dither_index] as usize
    }
}

#[cfg(test)]
mod tests {
    use color::LinearColor as LC;
    use super::{DitherPattern, Palette};
    use std::u16;

    const BLACK: LC = LC { r: 0, g: 0, b: 0 };
    const WHITE: LC = LC { r: u16::MAX, g: u16::MAX, b: u16::MAX };

    #[test]
    fn test_linear_color_squared_distance() {
        let gray = BLACK.lerp(WHITE, 0.5);
        assert_eq!(BLACK.squared_distance(BLACK), 0);
        assert!(BLACK.squared_distance(gray) < BLACK.squared_distance(WHITE));
    }

    #[test]
    fn test_linear_color_average() {
        assert_eq!(LC::average(&[BLACK, WHITE]), BLACK.lerp(WHITE, 0.5));
        assert_eq!(LC::average(&[BLACK, BLACK, WHITE]), BLACK.lerp(WHITE, 1.0/3.0));
    }

    #[test]
    fn test_palette_new() {
        let palette = Palette::new(2, &[BLACK, BLACK, WHITE, WHITE], false);
        assert_eq!(palette.colors.len(), 2);
        assert!(palette.colors.contains(&BLACK));
        assert!(palette.colors.contains(&WHITE));
    }

    #[test]
    fn test_palette_new_few_samples() {
        let palette = Palette::new(4, &[BLACK, WHITE], false);
        assert_eq!(palette.colors.len(), 4);
        assert_eq!(palette.colors.iter().filter(|&&c| c == BLACK).count(), 3);
        assert_eq!(palette.colors.iter().filter(|&&c| c == WHITE).count(), 1);
    }

    #[test]
    fn test_palette_get_dither_pattern() {
        let palette = Palette {
            colors: vec![BLACK, WHITE]
        };
        let di = palette.get_dither_pattern(LC::new_f32(0.5, 0.5, 0.5));
        assert_eq!(di.palette_indexes, [0, 1, 0, 0]);
        assert_eq!(di.palette_proportions, [32, 32, 0, 0]);
    }

    #[test]
    fn test_dither_pattern_get_palette_index() {
        fn test_proportions(proportions: [u8; 4]) {
            let d = DitherPattern {
                palette_indexes: [0, 1, 2, 3],
                palette_proportions: proportions
            };
            let mut counts = [0; 4];
            for x in 0..8 {
                for y in 0..8 {
                    counts[d.get_palette_index(x, y)] += 1;
                }
            }
            assert_eq!(proportions, counts, "Dithering did not produce expected proportions");
        }

        // Basic cases
        test_proportions([16, 16, 16, 16]);
        test_proportions([0, 32, 32, 0]);

        // Solid colors
        test_proportions([64, 0, 0, 0]);
        test_proportions([0, 64, 0, 0]);
        test_proportions([0, 0, 0, 64]);

        // 1:63
        test_proportions([1, 63, 0, 0]);
        test_proportions([1, 0, 63, 0]);
        test_proportions([63, 1, 0, 0]);
        test_proportions([63, 0, 1, 0]);
    }
}
