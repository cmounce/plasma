use std::f32;

pub trait FastMath<F> {
    fn wave(&self) -> F;
    fn cowave(&self) -> F;
    fn wrap(&self) -> F;
    fn lerp(&self, other: F, position: F) -> F;
    fn clamp(&self, lower: F, upper: F) -> F;
}

impl FastMath<f32> for f32 {
    // Like sin(), except its period is 1 instead of 2*PI
    #[inline]
    fn wave(&self) -> f32 {
        // x loops over the range [-0.5, 0.5).
        // Note that x is shifted by half a period: if self is 0, x is -0.5.
        let x = self - self.floor() - 0.5;

        /*
         * Approximate a sine wave shifted by half a period.
         *
         *     xxx  |
         *   xx   xx|
         * -x-------x-------x-
         *          |xx   xx
         *          |  xxx
         *
         * The interval (0.0, 0.5) is covered by the parabola f(x) = 16x^2 - 8x.
         * Having one x and one x.abs() flips the parabola upside-down when x is negative.
         */
        x * x.abs().mul_add(16.0, -8.0)
    }

    // Like cos(), except with a period of 1
    #[inline]
    fn cowave(&self) -> f32 {
        (self + 0.25).wave()
    }

    // Wraps a value onto the interval [0.0, 1.0).
    // For example, (-0.25).wrap() returns 0.75.
    #[inline]
    fn wrap(&self) -> f32 {
        self - self.floor()
    }

    // Linear interpolation from self to other
    fn lerp(&self, other: f32, position: f32) -> f32 {
        self*(1.0 - position) + other*position
    }

    // Restrict self to the specified range, inclusive
    fn clamp(&self, lower: f32, upper: f32) -> f32 {
        self.min(upper).max(lower)
    }
}

macro_rules! assert_feq {
    ($a:expr, $b:expr) => (
        {
            let a:f32 = $a;
            let b:f32 = $b;
            assert!((a - b).abs() < 0.01, "assertion failed: {} != {}", a, b);
        }
    );
}

#[cfg(test)]
mod tests {
    use fastmath::FastMath;

    #[test]
    fn test_wave() {
        // Test area around 0
        assert_feq!((-0.5).wave(), 0.0);
        assert_feq!((-0.25).wave(), -1.0);
        assert_feq!((0.0).wave(), 0.0);
        assert_feq!((0.25).wave(), 1.0);
        assert_feq!((0.5).wave(), 0.0);

        // Test area further away
        assert_feq!((8.0).wave(), 0.0);
        assert_feq!((8.25).wave(), 1.0);
        assert_feq!((8.5).wave(), 0.0);
        assert_feq!((8.75).wave(), -1.0);
        assert_feq!((9.0).wave(), 0.0);

        // Test area further into the negatives
        assert_feq!((-8.0).wave(), 0.0);
        assert_feq!((-7.75).wave(), 1.0);
        assert_feq!((-7.5).wave(), 0.0);
        assert_feq!((-7.25).wave(), -1.0);
        assert_feq!((-7.0).wave(), 0.0);
    }

    #[test]
    fn test_cowave() {
        assert_feq!((0.0).cowave(), 1.0);
        assert_feq!((0.25).cowave(), 0.0);
        assert_feq!((0.5).cowave(), -1.0);
        assert_feq!((0.75).cowave(), 0.0);
    }

    #[test]
    fn test_wrap() {
        // Non-negative inputs
        assert_feq!((0.0).wrap(), 0.0);
        assert_feq!((0.2).wrap(), 0.2);
        assert_feq!((1.0).wrap(), 0.0);
        assert_feq!((1.2).wrap(), 0.2);
        assert_feq!((7.2).wrap(), 0.2);

        // Negative inputs
        assert_feq!((-0.2).wrap(), 0.8);
        assert_feq!((-1.0).wrap(), 0.0);
        assert_feq!((-1.2).wrap(), 0.8);
        assert_feq!((-7.2).wrap(), 0.8);
    }

    #[test]
    fn test_lerp() {
        // Simple case
        assert_feq!((0.0).lerp(1.0, 0.0), 0.0);
        assert_feq!((0.0).lerp(1.0, 0.5), 0.5);
        assert_feq!((0.0).lerp(1.0, 1.0), 1.0);

        // Going backward
        assert_feq!((10.0).lerp(5.0, 0.2), 9.0);

        // Going past 0.0
        assert_feq!((5.0).lerp(-5.0, 0.2), 3.0);
        assert_feq!((5.0).lerp(-5.0, 0.5), 0.0);
        assert_feq!((5.0).lerp(-5.0, 0.8), -3.0);
    }

    #[test]
    fn test_clamp() {
        assert_feq!((-10.1).clamp(-10.0, 10.0), -10.0);
        assert_feq!((-10.0).clamp(-10.0, 10.0), -10.0);
        assert_feq!((5.0).clamp(-10.0, 10.0), 5.0);
        assert_feq!((10.0).clamp(-10.0, 10.0), 10.0);
        assert_feq!((10.1).clamp(-10.0, 10.0), 10.0);
    }
}
