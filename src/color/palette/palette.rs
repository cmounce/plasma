use cgmath::Vector3;
use cgmath::prelude::*;
use color::LinearColor;
use color::palette::dither::DitherPattern;
use ordered_float::OrderedFloat;
use std::u16;
use std::collections::HashSet;
use std::ops::Index;

pub struct Palette {
    pub colors: Vec<LinearColor>
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
        DitherPattern::new(color, &self)
    }
}

impl Index<usize> for Palette {
    type Output = LinearColor;

    fn index(&self, index: usize) -> &LinearColor {
        &self.colors[index]
    }
}

#[cfg(test)]
mod tests {
    use color::LinearColor as LC;
    use super::Palette;
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
}
