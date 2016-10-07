use fastmath::FastMath;
use genetics::{Chromosome, Gene};
use gradient::{Color, ControlPoint, Gradient};

const LOOKUP_TABLE_SIZE: usize = 256;
pub const CONTROL_POINT_GENE_SIZE: usize = 5;

impl Color {
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
}

impl ControlPoint {
    fn from_gene(gene: &Gene) -> Option<ControlPoint> {
        assert!(gene.data.len() == CONTROL_POINT_GENE_SIZE);
        let activation_threshold = 160;
        if gene.data[0] > activation_threshold {
            let h = (gene.data[1] as f32)/256.0; // disallow h = 1.0 (wraps to 0.0)
            let s = (gene.data[2] as f32)/255.0; // allow s = 1.0
            let l = (gene.data[3] as f32)/255.0; // allow l = 1.0
            let position = (gene.data[4] as f32)/256.0; // disallow position = 1.0 (wraps to 0.0)
            Some(ControlPoint {
                color: Color::from_hsl(h, s, l),
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
        let mut iter = gradient.iter();
        let mut subgradient = iter.next().unwrap();
        for i in 0..LOOKUP_TABLE_SIZE {
             let position = (i as f32)/(LOOKUP_TABLE_SIZE as f32);
             while !subgradient.contains(position) {
                 subgradient = iter.next().unwrap();
             }
             lookup_table[i] = subgradient.color_at(position);
        }

        ColorMapper {
            lookup_table: lookup_table
        }
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

    // Make sure max/min byte values map to different hues
    #[test]
    fn test_from_gene_hue() {
        let g1 = Gene { data: vec![255, 255, 255, 127, 255] };
        let g2 = Gene { data: vec![255,   0, 255, 127, 255] };
        let cp1 = ControlPoint::from_gene(&g1).unwrap();
        let cp2 = ControlPoint::from_gene(&g2).unwrap();
        assert!(cp1.color != cp2.color); // Make sure we have different hues
    }

    // Make sure full range of saturation is possible
    #[test]
    fn test_from_gene_saturation() {
        let g1 = Gene { data: vec![255, 0, 255, 127, 255] };
        let g2 = Gene { data: vec![255, 0,   0, 127, 255] };
        let cp1 = ControlPoint::from_gene(&g1).unwrap();
        let cp2 = ControlPoint::from_gene(&g2).unwrap();
        assert_eq!(cp1.color, Color::from_hsl(0.0, 1.0, 0.5));
        assert_eq!(cp2.color, Color::from_hsl(0.0, 0.0, 0.5));
    }

    // Make sure full range of value is possible
    #[test]
    fn test_from_gene_value() {
        let g1 = Gene { data: vec![255, 0, 255, 255, 255] };
        let g2 = Gene { data: vec![255, 0, 255,   0, 255] };
        let cp1 = ControlPoint::from_gene(&g1).unwrap();
        let cp2 = ControlPoint::from_gene(&g2).unwrap();
        assert_eq!(cp1.color, Color::from_hsl(0.0, 0.0, 1.0));
        assert_eq!(cp2.color, Color::from_hsl(0.0, 0.0, 0.0));
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
