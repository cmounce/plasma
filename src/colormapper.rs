use fastmath::FastMath;
use genetics::{Chromosome, Gene};
use gradient::{Color, ControlPoint, Gradient};
use std::f32;

const LOOKUP_TABLE_SIZE: usize = 512;
pub const NUM_COLOR_GENES: usize = 8;
pub const CONTROL_POINT_GENE_SIZE: usize = 5;

impl Color {
    fn sq_dist(&self, other: Color) -> u32 {
        // We intentionally do the distance calculation on gamma-encoded RGB values,
        // as opposed to doing it on linear values. Doing the calculation on gamma-encoded
        // values better matches perceived color distance. As a bonus, it's also faster.
        fn partial(x: u8, y: u8) -> u32 {
            let delta = (x as i32) - (y as i32);
            (delta*delta) as u32
        }
        partial(self.r, other.r) + partial(self.g, other.g) + partial(self.b, other.b)
    }

    fn avg(colors: &[Color]) -> Color {
        assert!(colors.len() > 0);
        macro_rules! avg_component {
            ($component:ident) => {{
                let gamma = 2.2;
                let linear_components = colors.iter().map(|c| (c.$component as f32).powf(gamma));
                let linear_sum: f32 = linear_components.sum();
                let linear_avg = linear_sum/colors.len() as f32;
                linear_avg.powf(1.0/gamma).round() as u8
            }};
        }
        Color {
            r: avg_component!(r),
            g: avg_component!(g),
            b: avg_component!(b)
        }
    }

    fn from_hsl(hue: f32, saturation: f32, lightness: f32) -> Color {
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

        // Helper function: convert linear f32 to gamma-correct u8 component
        fn to_component(linear: f32) -> u8 {
            let gamma = 2.2;
            let full = 255.0_f32.powf(gamma);
            (linear*full).powf(1.0/gamma).round() as u8
        }

        Color::new(
            to_component(r),
            to_component(g),
            to_component(b)
        )
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
    fn from_square_hsl(color_x: f32, color_y: f32, lightness: f32) -> Color {
        let x = (-1.0).lerp(1.0, color_x.clamp(0.0, 1.0));
        let y = (-1.0).lerp(1.0, color_y.clamp(0.0, 1.0));
        let saturation = x.abs().max(y.abs());
        if saturation == 0.0 {
            return Color::from_hsl(0.0, saturation, lightness);
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
        Color::from_hsl(hue, saturation, lightness)
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
                color: Color::from_square_hsl(color_x, color_y, lightness),
                position: position
            })
        } else {
            None
        }
    }
}

pub struct ColorMapper {
    lookup_table: [Color; LOOKUP_TABLE_SIZE]
}

impl ColorMapper {
    pub fn new(chromosome: &Chromosome) -> ColorMapper {
        let mut lookup_table = [Color {r:0, g:0, b:0}; LOOKUP_TABLE_SIZE];
        let mut control_points = vec![];
        for gene in chromosome.genes.iter() {
            if let Some(cp) = ControlPoint::from_gene(&gene) {
                control_points.push(cp);
            }
        }
        let gradient = Gradient::new(control_points);
        let palette = ColorMapper::calculate_palette(&gradient, 255);
        for i in 0..LOOKUP_TABLE_SIZE {
             let position = (i as f32)/(LOOKUP_TABLE_SIZE as f32);
             let color = gradient.get(position);
             let index = ColorMapper::quantize(color, &palette[..]);
             lookup_table[i] = palette[index as usize];
        }

        ColorMapper {
            lookup_table: lookup_table
        }
    }

    fn quantize(color: Color, palette: &[Color]) -> u8 {
        palette.iter().enumerate().min_by_key(|index_color|
            color.sq_dist(*index_color.1)
        ).unwrap().0 as u8
    }

    fn calculate_palette(gradient: &Gradient, palette_size: usize) -> Vec<Color> {
        assert!(palette_size >= 1);
        assert!(palette_size <= 256);

        // Sample many points on the gradient, more points than there are palette colors
        let num_samples = 512;
        let sample_step = 1.0/num_samples as f32;
        let mut samples = Vec::with_capacity(num_samples);
        for i in 0..num_samples {
            let position = sample_step*i as f32;
            samples.push(gradient.get(position));
        }

        // Create an initial palette by subsampling the samples Vec
        let mut palette = Vec::with_capacity(palette_size);
        let palette_step = 1.0/palette_size as f32;
        for i in 0..palette_size {
            palette.push(gradient.get(palette_step*i as f32));
        }

        // Do k-means clustering
        // The palette entries are the k means
        // The data points are the many samples we've taken of the gradient
        let mut quantized_samples = vec![0; num_samples];
        let mut palette_updated = true;
        while palette_updated {
            // Re-quantize our samples using the latest palette
            for i in 0..num_samples {
                quantized_samples[i] = ColorMapper::quantize(samples[i], &palette[..]);
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
                    let average = Color::avg(&palette_representees[i][..]);
                    if palette[i] != average {
                        palette[i] = average;
                        palette_updated = true;
                    }
                }
            }
        }

        palette
    }

    pub fn convert(&self, value: f32) -> Color {
        let index = (value.wrap()*(LOOKUP_TABLE_SIZE as f32)).floor() as usize % LOOKUP_TABLE_SIZE;
        self.lookup_table[index]
    }
}

#[cfg(test)]
mod tests {
    use genetics::Gene;
    use gradient::Color;
    use gradient::ControlPoint;

    #[test]
    fn test_color_sq_dist() {
        let black = Color::new(0, 0, 0);
        let white = Color::new(255, 255, 255);
        let gray = black.lerp(white, 0.5);
        assert_eq!(black.sq_dist(black), 0);
        assert!(black.sq_dist(gray) < black.sq_dist(white));
    }

    #[test]
    fn test_color_avg() {
        let black = Color::new(0, 0, 0);
        let white = Color::new(255, 255, 255);
        assert_eq!(Color::avg(&[black, white]), black.lerp(white, 0.5));
        assert_eq!(Color::avg(&[black, black, white]), black.lerp(white, 1.0/3.0));
    }

    #[test]
    fn test_color_from_hsl() {
        /*
         * H: red -> green -> blue
         * S: gray -> color
         * L: black -> color -> white
         */

        // Test saturated primaries and secondaries
        assert_eq!(Color::from_hsl(0.0, 1.0, 0.5), Color::new(255, 0, 0));
        assert_eq!(Color::from_hsl(1.0/6.0, 1.0, 0.5), Color::new(255, 255, 0));
        assert_eq!(Color::from_hsl(2.0/6.0, 1.0, 0.5), Color::new(0, 255, 0));
        assert_eq!(Color::from_hsl(3.0/6.0, 1.0, 0.5), Color::new(0, 255, 255));
        assert_eq!(Color::from_hsl(4.0/6.0, 1.0, 0.5), Color::new(0, 0, 255));
        assert_eq!(Color::from_hsl(5.0/6.0, 1.0, 0.5), Color::new(255, 0, 255));
        assert_eq!(Color::from_hsl(1.0, 1.0, 0.5), Color::new(255, 0, 0));

        // Test in-between colors
        let num_iter = 3*6;
        for i in 0..num_iter {
            let hue = i as f32/num_iter as f32;
            let sector = (6.0*hue).floor();
            let offset = (6.0*hue).fract();
            let previous = Color::from_hsl(sector/6.0, 1.0, 0.5);
            let next = Color::from_hsl((sector + 1.0)/6.0, 1.0, 0.5);
            assert_eq!(Color::from_hsl(hue, 1.0, 0.5), previous.lerp(next, offset));
        }

        // Test black, gray (gamma-correct), white
        let black = Color::new(0, 0, 0);
        let white = Color::new(255, 255, 255);
        let gray = black.lerp(white, 0.5);
        assert_eq!(Color::from_hsl(0.0, 0.0, 0.0), black);
        assert_eq!(Color::from_hsl(0.0, 1.0, 0.0), black);
        assert_eq!(Color::from_hsl(0.0, 0.0, 0.5), gray);
        assert_eq!(Color::from_hsl(0.0, 0.0, 1.0), white);
        assert_eq!(Color::from_hsl(0.0, 1.0, 1.0), white);

        // Test saturation
        let red = Color::new(255, 0, 0);
        assert_eq!(Color::from_hsl(0.0, 0.25, 0.5), gray.lerp(red, 0.25));
        assert_eq!(Color::from_hsl(0.0, 0.5,  0.5), gray.lerp(red, 0.5));
        assert_eq!(Color::from_hsl(0.0, 0.75, 0.5), gray.lerp(red, 0.75));
    }

    #[test]
    fn test_from_square_hsl() {
        // Test that going around the edge of the color square cycles through the hues
        assert_eq!(Color::from_square_hsl(0.0, 1.0, 0.5), Color::from_hsl(0.0/8.0, 1.0, 0.5));
        assert_eq!(Color::from_square_hsl(0.5, 1.0, 0.5), Color::from_hsl(1.0/8.0, 1.0, 0.5));
        assert_eq!(Color::from_square_hsl(1.0, 1.0, 0.5), Color::from_hsl(2.0/8.0, 1.0, 0.5));
        assert_eq!(Color::from_square_hsl(1.0, 0.5, 0.5), Color::from_hsl(3.0/8.0, 1.0, 0.5));
        assert_eq!(Color::from_square_hsl(1.0, 0.0, 0.5), Color::from_hsl(4.0/8.0, 1.0, 0.5));
        assert_eq!(Color::from_square_hsl(0.5, 0.0, 0.5), Color::from_hsl(5.0/8.0, 1.0, 0.5));
        assert_eq!(Color::from_square_hsl(0.0, 0.0, 0.5), Color::from_hsl(6.0/8.0, 1.0, 0.5));
        assert_eq!(Color::from_square_hsl(0.0, 0.5, 0.5), Color::from_hsl(7.0/8.0, 1.0, 0.5));

        // Test saturation
        assert_eq!(Color::from_square_hsl(0.5, 6.0/8.0, 0.5), Color::from_hsl(1.0/8.0, 0.5,  0.5));
        assert_eq!(Color::from_square_hsl(0.5, 5.0/8.0, 0.5), Color::from_hsl(1.0/8.0, 0.25, 0.5));
        assert_eq!(Color::from_square_hsl(0.5, 4.0/8.0, 0.5), Color::from_hsl(1.0/8.0, 0.0,  0.5));
        assert_eq!(Color::from_square_hsl(0.5, 3.0/8.0, 0.5), Color::from_hsl(5.0/8.0, 0.25, 0.5));
        assert_eq!(Color::from_square_hsl(0.5, 2.0/8.0, 0.5), Color::from_hsl(5.0/8.0, 0.5,  0.5));

        // Test lightness
        assert_eq!(Color::from_square_hsl(0.0, 1.0, 0.0),  Color::from_hsl(0.0, 1.0, 0.0));
        assert_eq!(Color::from_square_hsl(0.0, 1.0, 0.25), Color::from_hsl(0.0, 1.0, 0.25));
        assert_eq!(Color::from_square_hsl(0.0, 1.0, 0.75), Color::from_hsl(0.0, 1.0, 0.75));
        assert_eq!(Color::from_square_hsl(0.0, 1.0, 1.0),  Color::from_hsl(0.0, 1.0, 1.0));
    }

    // Make sure full ranges of chroma/lightness are possible
    #[test]
    fn test_from_gene_color() {
        fn to_color(data: [u8; 5]) -> Color {
            let g = Gene { data: data.to_vec() };
            let cp = ControlPoint::from_gene(&g).unwrap();
            return cp.color;
        }
        assert_eq!(to_color([255,   0,   0, 127, 255]), Color::from_square_hsl(-1.0, -1.0, 0.5));
        assert_eq!(to_color([255,   0, 255, 127, 255]), Color::from_square_hsl(-1.0,  1.0, 0.5));
        assert_eq!(to_color([255, 255,   0, 127, 255]), Color::from_square_hsl( 1.0, -1.0, 0.5));
        assert_eq!(to_color([255, 255, 255, 127, 255]), Color::from_square_hsl( 1.0,  1.0, 0.5));
        assert_eq!(to_color([255,   0,   0, 255, 255]), Color::from_square_hsl(-1.0, -1.0, 1.0));
        assert_eq!(to_color([255,   0,   0,   0, 255]), Color::from_square_hsl(-1.0, -1.0, 0.0));
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
