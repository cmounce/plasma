use fastmath::FastMath;

#[derive(Copy,Clone,Eq,PartialEq,Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Color {
        Color { r: r, g: g, b: b}
    }

    pub fn lerp(&self, other: Color, position: f32) -> Color {
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

#[derive(Copy,Clone,Debug)]
pub struct ControlPoint {
    pub color: Color,
    pub position: f32
}

impl ControlPoint {
    fn new(r: u8, g: u8, b: u8, position: f32) -> ControlPoint {
        ControlPoint {
            color: Color::new(r, g, b),
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
fn test_control_point_new() {
    let a = ControlPoint::new(1, 2, 3, 0.25);
    let b = ControlPoint::new(1, 2, 3, 1.25);
    let c = ControlPoint::new(1, 2, 3, -0.75);
    assert_eq!(a.position, b.position);
    assert_eq!(b.position, c.position);
}

#[test]
fn test_control_point_lerp() {
     /*
      * a    b        c    (a)
      * +----+--------+-----+
      * 0   0.2      0.7    1
      */
    let a = ControlPoint::new(60, 0, 0, 0.0);
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


#[derive(Debug)]
pub struct Subgradient {
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

    pub fn contains(&self, position: f32) -> bool {
        let adj_position = position.wrap();
        if self.point1.position <= self.point2.position {
            self.point1.position <= adj_position && adj_position <= self.point2.position
        } else {
            adj_position <= self.point2.position || self.point1.position <= adj_position
        }
    }

    pub fn color_at(&self, position: f32) -> Color {
        assert!(self.contains(position));
        self.point1.lerp(self.point2, position)
    }
}

#[test]
fn test_subgradient_contains() {
    let s = Subgradient::new(
        ControlPoint::new(0, 0, 0, 0.25),
        ControlPoint::new(0, 0, 0, 0.75)
    );
    assert!(!s.contains(0.24));
    assert!(s.contains(0.25));
    assert!(s.contains(0.5));
    assert!(s.contains(0.75));
    assert!(!s.contains(0.76));
}

#[test]
fn test_subgradient_contains_wraparound() {
    let s = Subgradient::new(
        ControlPoint::new(0, 0, 0, 0.75),
        ControlPoint::new(0, 0, 0, 0.25)
    );
    assert!(!s.contains(0.74));
    assert!(s.contains(0.75));
    assert!(s.contains(1.0));
    assert!(s.contains(1.25));
    assert!(!s.contains(1.26));
}

#[test]
fn test_subgradient_color_at() {
    let s = Subgradient::new(
        ControlPoint::new(60, 0, 0, 0.8),
        ControlPoint::new(0, 60, 0, 0.3),
    );
    assert_eq!(s.color_at(0.1), Color::new(24, 36, 0));
}


pub struct Gradient {
    points: Vec<ControlPoint>
}

impl Gradient {
    pub fn new(control_points: Vec<ControlPoint>) -> Gradient {
        assert!(control_points.len() >= 2);
        let mut points = control_points.clone();
        points.sort_by(|a, b| (a.position).partial_cmp(&b.position).unwrap());

        Gradient {
            points: points
        }
    }

    pub fn iter(&self) -> GradientIterator {
        GradientIterator {
            index1: self.points.len() - 1,
            gradient: &self
        }
    }
}


pub struct GradientIterator<'a> {
    index1: usize, // start index: index2 is index1 + 1
    gradient: &'a Gradient
}

impl<'a> Iterator for GradientIterator<'a> {
    type Item = Subgradient;

    fn next(&mut self) -> Option<Subgradient> {
        let index1 = self.index1;
        let index2 = (self.index1 + 1) % self.gradient.points.len();
        self.index1 = index2; // advance the iterator
        Some(Subgradient::new(self.gradient.points[index1], self.gradient.points[index2]))
    }
}
