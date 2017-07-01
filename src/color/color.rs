use cgmath::Vector3;

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
        let to_component = |f: f32| (f*65535.0).round() as u16;
        LinearColor::new(to_component(r), to_component(g), to_component(b))
    }

    pub fn new_vec3(v: &Vector3<f32>) -> LinearColor {
        LinearColor::new_f32(v.x, v.y, v.z)
    }

    pub fn to_vec3(&self) -> Vector3<f32> {
        Vector3 {
            x: self.r as f32,
            y: self.g as f32,
            z: self.b as f32
        } / 65535.0
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
        let anti_position = 1.0 - position;
        let lerp = |a, b| ((a as f32)*anti_position + (b as f32)*position).round() as u16;
        LinearColor {
            r: lerp(self.r, other.r),
            g: lerp(self.g, other.g),
            b: lerp(self.b, other.b),
        }
    }
}


#[cfg(test)]
mod tests {
    use std::u16;
    use super::{Color, LinearColor};

    #[test]
    fn test_linear_color_lerp() {
        // Test a basic color fade
        let a = LinearColor::new_f32(1.0, 0.0, 0.0);
        let b = LinearColor::new_f32(0.5, 0.5, 0.0);
        let c = LinearColor::new_f32(0.0, 1.0, 0.0);
        assert_eq!(a, a.lerp(c, 0.0));
        assert_eq!(b, a.lerp(c, 0.5));
        assert_eq!(c, a.lerp(c, 1.0));

        // Test rounding behavior
        let black = LinearColor::new(0, 0, 0);
        let dark_blue = LinearColor::new(0, 0, 2);
        let darker_blue = LinearColor::new(0, 0, 1);
        /*
         * When rounding, we want the most extreme colors to get half the space:
         *
         *   black    darker_blue   dark_blue
         * +-------+---------------+-------+
         * 0      0.25            0.75     1
         *
         * This is because in a multi-color gradient, the end of one color fade is always
         * the start of the next color fade, and so the extreme colors will appear twice.
         * But because they only get half the space, it all balances out.
         *
         * Is this important in the grand scheme of things? Probably not.
         */
        assert_eq!(black.lerp(dark_blue, 0.24), black);
        assert_eq!(black.lerp(dark_blue, 0.26), darker_blue);
        assert_eq!(black.lerp(dark_blue, 0.74), darker_blue);
        assert_eq!(black.lerp(dark_blue, 0.76), dark_blue);
    }

    #[test]
    fn test_linear_color_new_vec3() {
        let values = [0, 1, u16::MAX - 1, u16::MAX];
        for &value in values.iter() {
            let lc = LinearColor::new(value, value, value);
            let v = lc.to_vec3();
            assert_eq!(lc, LinearColor::new_vec3(&v));
        }
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
