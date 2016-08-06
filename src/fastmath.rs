use std::f32;

pub trait FastMath<F> {
    fn wave(&self) -> F;
    fn wrap(&self) -> F;
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

    // Wraps a value onto the interval [0.0, 1.0).
    // For example, (-0.25).wrap() returns 0.75.
    #[inline]
    fn wrap(&self) -> f32 {
        self - self.floor()
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
