use fastmath::FastMath;

const GAMMA: f32 = 2.2;

/**
 * Traditional 24-bit color, where each channel is gamma encoded.
 */
#[derive(Copy,Clone,Eq,PartialEq,Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

/**
 * A color representation where each channel is stored linearly (no gamma encoding).
 *
 * A LinearColor is 48 bits wide due to the lack of gamma encoding, but it is meant to cover
 * the same range as regular 24-bit color. In particular, it is possible to round-trip convert
 * a Color struct to LinearColor and back without loss of information.
 */
#[derive(Copy,Clone,Eq,PartialEq,Debug)]
pub struct LinearColor {
    pub r: u16,
    pub g: u16,
    pub b: u16
}


impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Color {
        Color { r: r, g: g, b: b }
    }

    pub fn to_linear(&self) -> LinearColor {
        LinearColor::from_gamma(*self)
    }
}

impl LinearColor {
    pub fn new(r: u16, g: u16, b: u16) -> LinearColor {
        LinearColor { r: r, g: g, b: b }
    }

    // Create a LinearColor with floats in the range [0.0, 1.0]
    pub fn new_f32(r: f32, g: f32, b: f32) -> LinearColor {
        /*
         * Note that to_component() uses (f*MAX+1).floor() instead of (f*MAX).round().
         *
         * For each u16 output, there is a range of f32 inputs that will produce that
         * output. Using floor() ensures that these input ranges are all the same size.
         *
         * Contrast with what would happen if we used round(). Inputs in the range [0.0, 0.5)
         * would map to 0, whereas inputs in the twice-as-large [0.5, 1.5) would map to 1.
         * Similarly, twice as many inputs would produce 65534 compared to 65535.
         */
        let to_component = |f: f32| (f*65536.0).floor().clamp(0.0, 65535.0) as u16;
        LinearColor::new(to_component(r), to_component(g), to_component(b))
    }

    fn component_to_linear(c: u8) -> u16 {
        let gamma_float = (c as f32)/255.0;
        let linear_float = gamma_float.powf(GAMMA);
        /*
         * Hack to fit a linear color component in a u16, while allowing round-trip conversion
         *
         * If we called round() to get the nearest u16, inputs 0 and 1 would have the same output:
         *      65535.0*(0.0/255.0)**2.2 = 0.0      (rounds to 0)
         *      65535.0*(1.0/255.0)**2.2 = 0.3327   (also rounds to 0)
         * This would result in loss of information: Color::new(1, 1, 1).to_linear() would return
         * the same thing as Color::new(0, 0, 0).to_linear().
         *
         * To avoid that, we call ceil() to get the nearest u16. Similarly, when we go in reverse
         * (linear to gamma), we call floor(). With a gamma of 2.2, this nudging-of-the-numbers is
         * just barely enough to avoid loss of information when doing round-trip conversions.
         */
        (linear_float*65535.0).ceil() as u16
    }

    fn component_to_gamma(c: u16) -> u8 {
        let linear_float = (c as f32)/65535.0;
        let gamma_float = linear_float.powf(1.0/GAMMA);
        (gamma_float*255.0).floor() as u8
    }

    pub fn to_gamma(&self) -> Color {
        Color {
            r: LinearColor::component_to_gamma(self.r),
            g: LinearColor::component_to_gamma(self.g),
            b: LinearColor::component_to_gamma(self.b)
        }
    }

    pub fn from_gamma(c: Color) -> LinearColor {
        LinearColor {
            r: LinearColor::component_to_linear(c.r),
            g: LinearColor::component_to_linear(c.g),
            b: LinearColor::component_to_linear(c.b)
        }
    }

    pub fn lerp(&self, other: LinearColor, position: f32) -> LinearColor {
        assert!(position >= 0.0 && position <= 1.0);
        LinearColor::new_f32(
            (self.r as f32).lerp(other.r as f32, position)/65535.0,
            (self.g as f32).lerp(other.g as f32, position)/65535.0,
            (self.b as f32).lerp(other.b as f32, position)/65535.0
        )
    }
}


#[cfg(test)]
mod tests {
    use super::{Color, LinearColor};

    #[test]
    fn test_linear_color_new_f32() {
        let black =         LinearColor::new(0, 0, 0);
        let almost_black =  LinearColor::new(1, 0, 0);
        let almost_red =    LinearColor::new(65534, 0, 0);
        let red =           LinearColor::new(65535, 0, 0);

        // There are 65536 possible u16 values.
        // Divide the range [0.0, 1.0] into 65536 ranges of equal size.
        let range_size = 1.0/65536.0;

        // Check rounding behavior at bottom
        let new_f32_r = |r: f32| LinearColor::new_f32(r, 0.0, 0.0);
        let a_little_bit = range_size/100.0;
        assert_eq!(black,           new_f32_r(0.0));
        assert_eq!(black,           new_f32_r(range_size - a_little_bit));
        assert_eq!(almost_black,    new_f32_r(range_size));
        assert_eq!(almost_black,    new_f32_r(2.0*range_size - a_little_bit));

        // Check rounding behavior at top
        assert_eq!(almost_red,  new_f32_r(1.0 - 2.0*range_size + a_little_bit));
        assert_eq!(almost_red,  new_f32_r(1.0 - range_size - a_little_bit));
        assert_eq!(red,         new_f32_r(1.0 - range_size + a_little_bit));
        assert_eq!(red,         new_f32_r(1.0));
    }

    #[test]
    fn test_linear_color_lerp() {
        let a = LinearColor::new_f32(1.0, 0.0, 0.0);
        let b = LinearColor::new_f32(0.5, 0.5, 0.0);
        let c = LinearColor::new_f32(0.0, 1.0, 0.0);
        assert_eq!(a, a.lerp(c, 0.0));
        assert_eq!(b, a.lerp(c, 0.5));
        assert_eq!(c, a.lerp(c, 1.0));
    }

    #[test]
    fn test_color_linearcolor_conversion() {
        // Test each channel
        let c = Color::new(85, 170, 255);
        assert_eq!(c, c.to_linear().to_gamma());

        // Test gamma calculation for 50% gray
        assert_eq!(
            LinearColor::new_f32(0.5, 0.5, 0.5).to_gamma(),
            Color::new(186, 186, 186)
        );

        // Test all values for a single channel, make sure we can round-trip to linear and back
        for i in 0..256 {
            let c = Color::new(i as u8, 0, 0);
            assert_eq!(c, c.to_linear().to_gamma());
        }
    }
}
