use color::LinearColor;
use color::palette::Palette;
use std::{cmp, u16};

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

// A dithering pattern that approximates a specific color
#[derive(Clone, Copy)]
pub struct DitherPattern {
    palette_indexes: [u16; 4],      // When dithering, mix these colors (up to 4)
    palette_proportions: [u8; 4]    // in these proportions (total of 64)
}

impl DitherPattern {
    pub fn new(color: LinearColor, palette: &Palette) -> DitherPattern {
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
                let palette_index = palette.get_nearest_index(target);
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
                subpalette.colors.push(palette[nearest_palette_index]);
                palette_indexes.push(nearest_palette_index);
                counts.push(1);
            }

            // Update our accumulated error
            let last_color = palette[nearest_palette_index];
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
    use color::LinearColor;
    use color::palette::Palette;
    use super::DitherPattern;

    #[test]
    fn test_dither_pattern_new() {
        let black = LinearColor::new_f32(0.0, 0.0, 0.0);
        let white = LinearColor::new_f32(1.0, 1.0, 1.0);
        let palette = Palette::new(2, &[black, white], false);
        let d = DitherPattern::new(LinearColor::new_f32(0.5, 0.5, 0.5), &palette);
        assert_eq!(d.palette_indexes, [0, 1, 0, 0]);
        assert_eq!(d.palette_proportions, [32, 32, 0, 0]);
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
