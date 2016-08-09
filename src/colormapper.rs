use fastmath::FastMath;
use std::cmp::Ordering;

const LOOKUP_TABLE_SIZE: usize = 256;

#[derive(Copy,Clone,Eq,PartialEq,Debug)]
struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

impl Color {
    fn new(r: u8, g: u8, b: u8) -> Color {
        Color { r: r, g: g, b: b}
    }

    fn lerp(&self, other: Color, position: f32) -> Color {
        assert!(position >= 0.0 && position <= 1.0);
        let opposite = 1.0 - position;
        Color {
            r: ((self.r as f32)*opposite + (other.r as f32)*position).round() as u8,
            g: ((self.g as f32)*opposite + (other.g as f32)*position).round() as u8,
            b: ((self.b as f32)*opposite + (other.b as f32)*position).round() as u8
        }
    }
}

#[test]
fn test_color_lerp() {
    let a = Color::new(0, 0, 0);
    let b = Color::new(128, 128, 128);
    assert_eq!(a, a.lerp(b, 0.0));
    assert_eq!(Color::new(64, 64, 64), a.lerp(b, 0.5));
    assert_eq!(b, a.lerp(b, 1.0));
}


#[derive(Copy,Clone)]
struct ControlPoint {
    color: Color,
    position: f32
}

impl ControlPoint {
    fn new(r: u8, g: u8, b: u8, position: f32) -> ControlPoint {
        ControlPoint {
            color: Color {r: r, g: g, b: b},
            position: position.wrap()
        }
    }

    fn lerp(&self, other: ControlPoint, position: f32) -> Color {
        // Calculate distance from self to other, moving in the positive direction
        let distance = (other.position - self.position).wrap();
        assert!(distance > 0.0);
        let adj_position = (position - self.position).wrap()/distance;
        self.color.lerp(other.color, adj_position)
    }
}

#[test]
fn test_control_point_lerp() {
     /*
      * a    b        c    (a)
      * +----+--------+-----+
      * 0   0.2      0.7    1
      */
    let a = ControlPoint::new(60, 0, 0, 1.0); // 1.0 should wrap to 0.0
    let b = ControlPoint::new(0, 60, 0, 0.2);
    let c = ControlPoint::new(0, 0, 60, 0.7);

    // Test interval starting at 0.0/1.0
    assert_eq!(a.lerp(b, 0.0), Color::new(60, 0, 0));
    assert_eq!(a.lerp(b, 0.1), Color::new(30, 30, 0));
    assert_eq!(a.lerp(b, 0.2), Color::new(0, 60, 0));

    // Test middle interval
    assert_eq!(b.lerp(c, 0.2), Color::new(0, 60, 0));
    assert_eq!(b.lerp(c, 0.3), Color::new(0, 48, 12));
    assert_eq!(b.lerp(c, 0.7), Color::new(0, 0, 60));

    // Test interval ending at 0.0/1.0
    assert_eq!(c.lerp(a, 0.7), Color::new(0, 0, 60));
    assert_eq!(c.lerp(a, 0.8), Color::new(20, 0, 40));
    assert_eq!(c.lerp(a, 1.0), Color::new(60, 0, 0));

    // Test interval crossing 0.0/1.0
    assert_eq!(c.lerp(b, 0.7), Color::new(0, 0, 60));
    assert_eq!(c.lerp(b, 0.8), Color::new(0, 12, 48));
    assert_eq!(c.lerp(b, 0.0), Color::new(0, 36, 24));
    assert_eq!(c.lerp(b, 0.1), Color::new(0, 48, 12));
    assert_eq!(c.lerp(b, 0.2), Color::new(0, 60, 0));
}


struct Subgradient {
    point1: ControlPoint,
    point2: ControlPoint
}

impl Subgradient {
    fn new(point1: ControlPoint, point2: ControlPoint) -> Subgradient {
        Subgradient {
            point1: point1,
            point2: point2
        }
    }

    fn contains(&self, position: f32) -> bool {
        let adj_position = position.wrap();
        let naive_contains =
            self.point1.position <= adj_position &&
            adj_position <= self.point2.position;
        if self.point1.position <= self.point2.position {
            naive_contains
        } else {
            !naive_contains
        }
    }
}

#[test]
fn test_subgradient_contains() {
    let s = Subgradient::new(
        ControlPoint::new(0, 0, 0, 0.2),
        ControlPoint::new(0, 0, 0, 0.7)
    );
    assert!(!s.contains(0.1));
    assert!(s.contains(0.2));
    assert!(s.contains(0.5));
    assert!(s.contains(0.7));
    assert!(!s.contains(0.8));
}

#[test]
#[ignore] // TODO fix
fn test_subgradient_contains_wraparound() {
    let s = Subgradient::new(
        ControlPoint::new(0, 0, 0, 0.7),
        ControlPoint::new(0, 0, 0, 0.2)
    );
    assert!(!s.contains(0.6));
    assert!(s.contains(0.7));
    assert!(s.contains(1.0));
    assert!(s.contains(1.2));
    assert!(!s.contains(1.3));
}


struct Gradient {
    points: Vec<ControlPoint>
}

impl Gradient {
    fn new() -> Gradient {
        let mut points = vec![
            ControlPoint::new(0, 32, 64, 0.0),
            ControlPoint::new(64, 96, 192, 0.5)
        ];
        points.sort_by(|a, b| (a.position).partial_cmp(&b.position).unwrap());
        assert!(points.len() >= 2);

        Gradient {
            points: points
        }
    }

    fn iter(&self) -> GradientIterator {
        GradientIterator {
            index1: self.points.len() - 1,
            gradient: &self
        }
    }
}


struct GradientIterator<'a> {
    index1: usize,
    gradient: &'a Gradient
}

impl<'a> Iterator for GradientIterator<'a> {
    type Item = (ControlPoint, ControlPoint);

    fn next(&mut self) -> Option<(ControlPoint, ControlPoint)> {
        let index1 = self.index1;
        let index2 = (self.index1 + 1) % self.gradient.points.len();
        self.index1 = index2; // advance the iterator
        Some((self.gradient.points[index1], self.gradient.points[index1]))
    }
}


pub struct ColorMapper {
    color1: Color,
    color2: Color,
    lookup_table: [Color; LOOKUP_TABLE_SIZE]
}

impl ColorMapper {
    pub fn new() -> ColorMapper {
        ColorMapper {
            color1: Color {r:0, g:32, b:64},
            color2: Color {r:64, g:96, b:192},
            lookup_table: [Color {r:0, g:0, b:0}; LOOKUP_TABLE_SIZE]
        }
    }

    fn compute_lookup_table(&mut self) {
        let gradient = Gradient::new();
        let mut iter = gradient.iter();
        for i in 0..LOOKUP_TABLE_SIZE {
            let position = (i as f32)/(LOOKUP_TABLE_SIZE as f32);
            self.lookup_table[i] = Color {r:0, g:0, b:0};
        }
    }

    pub fn convert(&self, value: f32) -> (u8, u8, u8) {
        let value_adj = value - value.floor();
        let position = (value_adj - 0.5).abs()*2.0;
        let opposite = 1.0 - position;
        (
            ((self.color1.r as f32)*position + (self.color2.r as f32)*opposite) as u8,
            ((self.color1.g as f32)*position + (self.color2.g as f32)*opposite) as u8,
            ((self.color1.b as f32)*position + (self.color2.b as f32)*opposite) as u8
        )
    }
}
