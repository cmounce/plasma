use fastmath::FastMath;
use genetics::{Chromosome, Gene};
use gradient::{Color, ControlPoint, Gradient, LinearColor};
use settings::RenderingSettings;
use std::{f32, u16};

const LOOKUP_TABLE_SIZE: usize = 512;
pub const NUM_COLOR_GENES: usize = 8;
pub const CONTROL_POINT_GENE_SIZE: usize = 5;

impl LinearColor {
    fn from_hsl(hue: f32, saturation: f32, lightness: f32) -> LinearColor {
        let h = hue.wrap();
        let s = saturation.clamp(0.0, 1.0);
        let l = lightness.clamp(0.0, 1.0);

        // Calculate upper and lower bounds on color components, based at lightness = 50%
        let upper_l50 = 0.5 + s/2.0;
        let lower_l50 = 0.5 - s/2.0;

        // Calculate upper and lower bounds with actual lightness applied
        let black_white = l.round();
        let position = (l - 0.5).abs()*2.0;
        let upper = upper_l50.lerp(black_white, position);
        let lower = lower_l50.lerp(black_white, position);

        // Calculate component values based on upper/lower bounds, hue
        let sector = (h*6.0) as u32 % 6;
        let offset = (h*6.0).fract();
        let (r, g, b) = match sector {
            0 => (upper, lower.lerp(upper, offset), lower),
            1 => (upper.lerp(lower, offset), upper, lower),
            2 => (lower, upper, lower.lerp(upper, offset)),
            3 => (lower, upper.lerp(lower, offset), upper),
            4 => (lower.lerp(upper, offset), lower, upper),
            5 => (upper, lower, upper.lerp(lower, offset)),
            _ => panic!("Invalid sector value {}", sector)
        };
        LinearColor::new_f32(r, g, b)
    }

    /*
     * A transformed version of HSL whose coordinates are Cartesian rather than cylindrical.
     *
     * - color_x and color_y are Cartesian coordinates on a square color wheel:
     *      - (0.0, 1.0) is the upper-left corner of the square (H = 0.0, S = 1.0)
     *      - (1.0, 0.7) is 1/4 + 3/40 clockwise around the perimeter (H = 0.325, S = 1.0)
     *      - (0.5, 0.5) is the center of the square (S = 0.0)
     * - lightness goes from 0.0 to 1.0, and works the same as in regular HSL
     */
    fn from_square_hsl(color_x: f32, color_y: f32, lightness: f32) -> LinearColor {
        let x = (-1.0).lerp(1.0, color_x.clamp(0.0, 1.0));
        let y = (-1.0).lerp(1.0, color_y.clamp(0.0, 1.0));
        let saturation = x.abs().max(y.abs());
        if saturation == 0.0 {
            return LinearColor::from_hsl(0.0, saturation, lightness);
        }

        let side_length = saturation*2.0;
        let perimeter = side_length*4.0;
        let adj_x = (x + saturation)/perimeter;
        let adj_y = (y + saturation)/perimeter;
        let hue = match (y > x, y > -x) {
            (true,  true)  => adj_x,
            (false, true)  => 0.25 + (0.25 - adj_y),
            (false, false) => 0.5 + (0.25 - adj_x),
            (true,  false) => 0.75 + adj_y
        };
        LinearColor::from_hsl(hue, saturation, lightness)
    }

    fn sq_dist(&self, other: LinearColor) -> u64 {
        fn partial(x: u16, y: u16) -> u64 {
            let delta = (x as i64) - (y as i64);
            (delta*delta) as u64
        }
        partial(self.r, other.r) + partial(self.g, other.g) + partial(self.b, other.b)
    }
}

trait PaletteUtils {
    fn average(&self) -> LinearColor;
    fn get_nearest_palette_index(&self, color: LinearColor) -> usize;
}

impl PaletteUtils for [LinearColor] {
    fn average(&self) -> LinearColor {
        assert!(self.len() > 0);
        let totals = self.iter().fold([0.0, 0.0, 0.0], |acc, c| {
            [acc[0] + c.r as f32, acc[1] + c.g as f32, acc[2] + c.b as f32]
        });
        let avg_component = |total: f32| (total/(self.len() as f32)).round() as u16;
        LinearColor {
            r: avg_component(totals[0]),
            g: avg_component(totals[1]),
            b: avg_component(totals[2])
        }
    }

    // Given a palette and an arbitrary color, returns the index of the nearest palette color
    fn get_nearest_palette_index(&self, color: LinearColor) -> usize {
        self.iter().enumerate().min_by_key(|&(_, palette_color)|
            color.sq_dist(*palette_color)
        ).unwrap().0
    }
}

impl ControlPoint {
    fn from_gene(gene: &Gene) -> Option<ControlPoint> {
        assert!(gene.data.len() == CONTROL_POINT_GENE_SIZE);
        let activation_threshold = 140;
        if gene.data[0] > activation_threshold {
            let color_x = (gene.data[1] as f32)/255.0; // allow color_x = 1.0
            let color_y = (gene.data[2] as f32)/255.0; // allow color_y = 1.0
            let lightness = (gene.data[3] as f32)/255.0; // allow lightness = 1.0
            let position = (gene.data[4] as f32)/256.0; // disallow position = 1.0 (wraps to 0.0)
            Some(ControlPoint {
                color: LinearColor::from_square_hsl(color_x, color_y, lightness),
                position: position
            })
        } else {
            None
        }
    }
}

pub struct ColorMapper {
    palette: Vec<Color>,
    lookup_table: [u16; LOOKUP_TABLE_SIZE]
}

impl ColorMapper {
    pub fn new(chromosome: &Chromosome, settings: &RenderingSettings) -> ColorMapper {
        // Build gradient
        let control_points = chromosome.genes.iter().
            filter_map(|g| ControlPoint::from_gene(&g)).collect();
        let gradient = Gradient::new(control_points);

        // Compute optimal palette for gradient
        let linear_palette = ColorMapper::calculate_palette(
            &gradient,
            settings.palette_size.unwrap_or(LOOKUP_TABLE_SIZE)
        );

        // Build gradient-position -> palette-index lookup table
        let mut lookup_table = [0; LOOKUP_TABLE_SIZE];
        for i in 0..LOOKUP_TABLE_SIZE {
            let position = (i as f32)/(LOOKUP_TABLE_SIZE as f32);
            let color = gradient.get_color(position);
            lookup_table[i] = linear_palette.get_nearest_palette_index(color) as u16;
        }

        // Gamma-encode palette and return finished ColorMapper
        ColorMapper {
            palette: linear_palette.iter().map(|lc| lc.to_gamma()).collect(),
            lookup_table: lookup_table
        }
    }

    fn calculate_palette(gradient: &Gradient, palette_size: usize) -> Vec<LinearColor> {
        assert!(palette_size >= 2);
        assert!(palette_size <= u16::MAX as usize);

        // Sample many points on the gradient, more points than there are palette colors
        let num_samples = LOOKUP_TABLE_SIZE;
        let sample_step = 1.0/num_samples as f32;
        let mut samples = Vec::with_capacity(num_samples);
        for i in 0..num_samples {
            let position = sample_step*i as f32;
            samples.push(gradient.get_color(position));
        }

        // Create an initial palette by sampling the gradient
        let mut palette = Vec::with_capacity(palette_size);
        let palette_step = 1.0/palette_size as f32;
        for i in 0..palette_size {
            palette.push(gradient.get_color(palette_step*i as f32));
        }

        // Do k-means clustering
        // The palette entries are the k means
        // The data points are the many samples we've taken of the gradient
        let mut quantized_samples = vec![0; num_samples];
        let mut palette_updated = true;
        while palette_updated {
            // Re-quantize our samples using the latest palette
            for i in 0..num_samples {
                quantized_samples[i] = palette.get_nearest_palette_index(samples[i]) as u16;
            }

            // Rebuild our palette
            palette_updated = false;
            let mut palette_representees = vec![vec![]; palette_size];
            for i in 0..num_samples {
                let palette_index = quantized_samples[i] as usize;
                palette_representees[palette_index].push(samples[i]);
            }
            for i in 0..palette_size {
                if palette_representees[i].len() > 0 {
                    let average = palette_representees[i].average();
                    if palette[i] != average {
                        palette[i] = average;
                        palette_updated = true;
                    }
                }
            }
        }

        palette
    }

    /*
     * TODO: Add a corresponding get_dithered_color(position, x, y)
     * We can use Yliluoma's algorithm. 4 color max, 8x8 Bayer matrix
     * We'll precompute dither information (indexes of base colors + their proportions)
     */
    pub fn get_nearest_color(&self, value: f32) -> Color {
        let index = (value.wrap()*(LOOKUP_TABLE_SIZE as f32)).floor() as usize % LOOKUP_TABLE_SIZE;
        let palette_index = self.lookup_table[index];
        self.palette[palette_index as usize]
    }

    pub fn get_palette(&self) -> Vec<Color> {
        self.palette.clone()
    }
}

#[cfg(test)]
mod tests {
    use genetics::Gene;
    use gradient::{Color, LinearColor as LC};
    use gradient::ControlPoint;
    use super::PaletteUtils;

    // Create a LinearColor with gamma-encoded u8 values
    fn new_gamma(r: u8, g: u8, b: u8) -> LC {
        Color::new(r, g, b).to_linear()
    }

    #[test]
    fn test_linear_color_sq_dist() {
        let black = new_gamma(0, 0, 0);
        let white = new_gamma(255, 255, 255);
        let gray = black.lerp(white, 0.5);
        assert_eq!(black.sq_dist(black), 0);
        assert!(black.sq_dist(gray) < black.sq_dist(white));
    }

    #[test]
    fn test_palette_utils_average() {
        let black = new_gamma(0, 0, 0);
        let white = new_gamma(255, 255, 255);
        assert_eq!([black, white].average(), black.lerp(white, 0.5));
        assert_eq!([black, black, white].average(), black.lerp(white, 1.0/3.0));
    }

    #[test]
    fn test_linear_color_from_hsl() {
        /*
         * H: red -> green -> blue
         * S: gray -> color
         * L: black -> color -> white
         */

        // Test saturated primaries and secondaries
        assert_eq!(LC::from_hsl(0.0,     1.0, 0.5), new_gamma(255, 0,   0));
        assert_eq!(LC::from_hsl(1.0/6.0, 1.0, 0.5), new_gamma(255, 255, 0));
        assert_eq!(LC::from_hsl(2.0/6.0, 1.0, 0.5), new_gamma(0,   255, 0));
        assert_eq!(LC::from_hsl(3.0/6.0, 1.0, 0.5), new_gamma(0,   255, 255));
        assert_eq!(LC::from_hsl(4.0/6.0, 1.0, 0.5), new_gamma(0,   0,   255));
        assert_eq!(LC::from_hsl(5.0/6.0, 1.0, 0.5), new_gamma(255, 0,   255));
        assert_eq!(LC::from_hsl(1.0,     1.0, 0.5), new_gamma(255, 0,   0));

        // Test in-between colors
        let num_iter = 3*6;
        for i in 0..num_iter {
            let hue = i as f32/num_iter as f32;
            let sector = (6.0*hue).floor();
            let offset = (6.0*hue).fract();
            let previous = LC::from_hsl(sector/6.0, 1.0, 0.5);
            let next = LC::from_hsl((sector + 1.0)/6.0, 1.0, 0.5);
            assert_eq!(LC::from_hsl(hue, 1.0, 0.5), previous.lerp(next, offset));
        }

        // Test black, gray (gamma-correct), white
        let black = new_gamma(0, 0, 0);
        let white = new_gamma(255, 255, 255);
        let gray = black.lerp(white, 0.5);
        assert_eq!(LC::from_hsl(0.0, 0.0, 0.0), black);
        assert_eq!(LC::from_hsl(0.0, 1.0, 0.0), black);
        assert_eq!(LC::from_hsl(0.0, 0.0, 0.5), gray);
        assert_eq!(LC::from_hsl(0.0, 0.0, 1.0), white);
        assert_eq!(LC::from_hsl(0.0, 1.0, 1.0), white);

        // Test saturation
        let red = new_gamma(255, 0, 0);
        assert_eq!(LC::from_hsl(0.0, 0.25, 0.5), gray.lerp(red, 0.25));
        assert_eq!(LC::from_hsl(0.0, 0.5,  0.5), gray.lerp(red, 0.5));
        assert_eq!(LC::from_hsl(0.0, 0.75, 0.5), gray.lerp(red, 0.75));
    }

    #[test]
    fn test_from_square_hsl() {
        // Test that going around the edge of the color square cycles through the hues
        assert_eq!(LC::from_square_hsl(0.0, 1.0, 0.5), LC::from_hsl(0.0/8.0, 1.0, 0.5));
        assert_eq!(LC::from_square_hsl(0.5, 1.0, 0.5), LC::from_hsl(1.0/8.0, 1.0, 0.5));
        assert_eq!(LC::from_square_hsl(1.0, 1.0, 0.5), LC::from_hsl(2.0/8.0, 1.0, 0.5));
        assert_eq!(LC::from_square_hsl(1.0, 0.5, 0.5), LC::from_hsl(3.0/8.0, 1.0, 0.5));
        assert_eq!(LC::from_square_hsl(1.0, 0.0, 0.5), LC::from_hsl(4.0/8.0, 1.0, 0.5));
        assert_eq!(LC::from_square_hsl(0.5, 0.0, 0.5), LC::from_hsl(5.0/8.0, 1.0, 0.5));
        assert_eq!(LC::from_square_hsl(0.0, 0.0, 0.5), LC::from_hsl(6.0/8.0, 1.0, 0.5));
        assert_eq!(LC::from_square_hsl(0.0, 0.5, 0.5), LC::from_hsl(7.0/8.0, 1.0, 0.5));

        // Test saturation
        assert_eq!(LC::from_square_hsl(0.5, 6.0/8.0, 0.5), LC::from_hsl(1.0/8.0, 0.5,  0.5));
        assert_eq!(LC::from_square_hsl(0.5, 5.0/8.0, 0.5), LC::from_hsl(1.0/8.0, 0.25, 0.5));
        assert_eq!(LC::from_square_hsl(0.5, 4.0/8.0, 0.5), LC::from_hsl(1.0/8.0, 0.0,  0.5));
        assert_eq!(LC::from_square_hsl(0.5, 3.0/8.0, 0.5), LC::from_hsl(5.0/8.0, 0.25, 0.5));
        assert_eq!(LC::from_square_hsl(0.5, 2.0/8.0, 0.5), LC::from_hsl(5.0/8.0, 0.5,  0.5));

        // Test lightness
        assert_eq!(LC::from_square_hsl(0.0, 1.0, 0.0),  LC::from_hsl(0.0, 1.0, 0.0));
        assert_eq!(LC::from_square_hsl(0.0, 1.0, 0.25), LC::from_hsl(0.0, 1.0, 0.25));
        assert_eq!(LC::from_square_hsl(0.0, 1.0, 0.75), LC::from_hsl(0.0, 1.0, 0.75));
        assert_eq!(LC::from_square_hsl(0.0, 1.0, 1.0),  LC::from_hsl(0.0, 1.0, 1.0));
    }

    // Make sure full ranges of chroma/lightness are possible
    #[test]
    fn test_from_gene_color() {
        fn to_color(data: [u8; 5]) -> LC {
            let g = Gene { data: data.to_vec() };
            let cp = ControlPoint::from_gene(&g).unwrap();
            cp.color
        }
        let half = 127.0/255.0; // Exactly 50% lightness cannot be expressed, because 255 is odd
        assert_eq!(to_color([255,   0,   0, 127, 255]), LC::from_square_hsl(0.0, 0.0, half));
        assert_eq!(to_color([255,   0, 255, 127, 255]), LC::from_square_hsl(0.0, 1.0, half));
        assert_eq!(to_color([255, 255,   0, 127, 255]), LC::from_square_hsl(1.0, 0.0, half));
        assert_eq!(to_color([255, 255, 255, 127, 255]), LC::from_square_hsl(1.0, 1.0, half));
        assert_eq!(to_color([255,   0,   0, 255, 255]), LC::from_square_hsl(0.0, 0.0, 1.0));
        assert_eq!(to_color([255,   0,   0,   0, 255]), LC::from_square_hsl(0.0, 0.0, 0.0));
    }

    // Make sure max/min byte values map to different positions
    #[test]
    fn test_from_gene_position() {
        let g1 = Gene { data: vec![255, 255, 255, 255, 255] };
        let g2 = Gene { data: vec![255, 255, 255, 255,   0] };
        let cp1 = ControlPoint::from_gene(&g1).unwrap();
        let cp2 = ControlPoint::from_gene(&g2).unwrap();
        assert!(cp1.position != cp2.position);
    }
}
