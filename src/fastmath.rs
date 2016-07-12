use std::f32;

pub trait FastMath<F> {
    fn xwave(&self) -> F;
    fn ywave(&self) -> F;
}

impl FastMath<f32> for f32 {
    // Like sin(), except its period is 10 instead of 2*PI
    fn ywave(&self) -> f32 {
        // x loops over the range (-0.5, 0.5)
        let adj = self/10.0;
        let x = adj - adj.floor() - 0.5;

        // f(x) = 16x^2 - 8x, a parabola centered on the interval (0.0, 0.5).
        // Having one x and one x.abs() flips the parabola when x is negative.
        x * x.abs().mul_add(16.0, -8.0)
    }

    // Like cos(), except its period is 10 instead of 2*PI
    fn xwave(&self) -> f32 {
        return (self + 2.5).ywave();
    }
}

macro_rules! assert_feq {
    ($expected:expr, $actual:expr) => (
        {
            let expected:f32 = $expected;
            let actual:f32 = $actual;
            assert!(expected.abs_sub(actual) < 0.01, "assertion failed: {} != {}", expected, actual);
        }
    );
}

#[test]
fn test_ywave() {
    // Test area around 0
    assert_feq!((-0.5).ywave(), 0.0);
    assert_feq!((-0.25).ywave(), -1.0);
    assert_feq!((0.0).ywave(), 0.0);
    assert_feq!((0.25).ywave(), 1.0);
    assert_feq!((0.5).ywave(), 0.0);

    // Test area further away
    assert_feq!((8.0).ywave(), 0.0);
    assert_feq!((8.25).ywave(), 1.0);
    assert_feq!((8.5).ywave(), 0.0);
    assert_feq!((8.75).ywave(), -1.0);
    assert_feq!((9.0).ywave(), 0.0);

    // Test area further into the negatives
    assert_feq!((-8.0).ywave(), 0.0);
    assert_feq!((-7.75).ywave(), 1.0);
    assert_feq!((-7.5).ywave(), 0.0);
    assert_feq!((-7.25).ywave(), -1.0);
    assert_feq!((-7.0).ywave(), 0.0);
}
