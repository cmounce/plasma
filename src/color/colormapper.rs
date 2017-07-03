use color::{Color, LinearColor};
use color::gradient::{ControlPoint, Gradient};
use color::palette::{DitherPattern, Palette};
use fastmath::FastMath;
use genetics::{Chromosome, Gene};
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
    gamma_palette: Vec<Color>,
    lookup_table_nearest: Vec<u16>,
    lookup_table_dithered: Vec<DitherPattern>
}

impl ColorMapper {
    pub fn new(chromosome: &Chromosome, settings: &RenderingSettings) -> ColorMapper {
        // Build gradient and sample it
        let control_points = chromosome.genes.iter().
            filter_map(|g| ControlPoint::from_gene(&g)).collect();
        let gradient = Gradient::new(control_points);
        let sample_step = 1.0/LOOKUP_TABLE_SIZE as f32;
        let sample_positions = (0..LOOKUP_TABLE_SIZE).map(|i| i as f32*sample_step);
        let gradient_samples: Vec<_> = sample_positions.map(|p| gradient.get_color(p)).collect();

        // Build a palette from the gradient samples
        let palette_size = settings.palette_size.unwrap_or(LOOKUP_TABLE_SIZE);
        let palette = Palette::new(palette_size, &gradient_samples, settings.dithering);

        // Use the samples and the palette to build lookup tables
        let mut lookup_table_nearest = vec![];
        let mut lookup_table_dithered = vec![];
        if settings.dithering {
            // Build gradient-position -> precomputed-dither-pattern lookup table
            lookup_table_dithered = gradient_samples.iter().map(
                |&color| palette.get_dither_pattern(color)
            ).collect();
        } else {
            // Build gradient-position -> nearest-palette-index lookup table
            lookup_table_nearest = gradient_samples.iter().map(
                |&color| palette.get_nearest_index(color) as u16
            ).collect();
        }

        // Gamma-encode palette and return finished ColorMapper
        ColorMapper {
            gamma_palette: palette.colors.iter().map(|color| color.to_gamma()).collect(),
            lookup_table_nearest: lookup_table_nearest,
            lookup_table_dithered: lookup_table_dithered
        }
    }

    pub fn get_nearest_color(&self, position: f32) -> Color {
        assert!(!self.lookup_table_nearest.is_empty(), "ColorMapper created with dithering on");
        let float_index = (position.wrap()*(LOOKUP_TABLE_SIZE as f32)).floor();
        let index = (float_index as usize) % LOOKUP_TABLE_SIZE;
        let palette_index = self.lookup_table_nearest[index];
        self.gamma_palette[palette_index as usize]
    }

    pub fn get_dithered_color(&self, position: f32, x: usize, y: usize) -> Color {
        assert!(!self.lookup_table_dithered.is_empty(), "ColorMapper created with dithering off");
        let float_index = (position.wrap()*(LOOKUP_TABLE_SIZE as f32)).floor();
        let index = (float_index as usize) % LOOKUP_TABLE_SIZE;
        let dither_info = self.lookup_table_dithered[index];
        let palette_index = dither_info.get_palette_index(x, y);
        self.gamma_palette[palette_index]
    }

    pub fn get_palette(&self) -> Vec<Color> {
        self.gamma_palette.clone()
    }
}

#[cfg(test)]
mod tests {
    use genetics::Gene;
    use cgmath::Vector3;
    use cgmath::prelude::*;
    use color::{Color, LinearColor as LC};
    use color::gradient::ControlPoint;

    // Create a LinearColor with gamma-encoded u8 values
    fn new_gamma(r: u8, g: u8, b: u8) -> LC {
        Color::new(r, g, b).to_linear()
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
        macro_rules! assert_close {
            ($a:expr, $b:expr) => (
                {
                    let a: LC = $a;
                    let b: LC = $b;
                    let diff: Vector3<f32> = a.to_vec3() - b.to_vec3();
                    assert!(diff.magnitude() < 0.01, "assertion failed: {:?} != {:?}", a, b);
                }
            );
        }
        let num_iter = 3*6;
        for i in 0..num_iter {
            let hue = i as f32/num_iter as f32;
            let sector = (6.0*hue).floor();
            let offset = (6.0*hue).fract();
            let previous = LC::from_hsl(sector/6.0, 1.0, 0.5);
            let next = LC::from_hsl((sector + 1.0)/6.0, 1.0, 0.5);
            assert_close!(LC::from_hsl(hue, 1.0, 0.5), previous.lerp(next, offset));
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
        assert_close!(LC::from_hsl(0.0, 0.25, 0.5), gray.lerp(red, 0.25));
        assert_close!(LC::from_hsl(0.0, 0.5,  0.5), gray.lerp(red, 0.5));
        assert_close!(LC::from_hsl(0.0, 0.75, 0.5), gray.lerp(red, 0.75));
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
