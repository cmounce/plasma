use fastmath::FastMath;

#[derive(Copy,Clone,Eq,PartialEq,Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

#[derive(Copy,Clone,Debug)]
pub struct ControlPoint {
    pub color: Color,
    pub position: f32
}

#[derive(Debug)]
pub struct Subgradient {
    point1: ControlPoint,
    point2: ControlPoint
}

pub struct Gradient {
    points: Vec<ControlPoint>
}

pub struct GradientIterator<'a> {
    index1: usize, // start index: index2 is index1 + 1
    gradient: &'a Gradient
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Color {
        Color { r: r, g: g, b: b}
    }

    pub fn lerp(&self, other: Color, position: f32) -> Color {
        assert!(position >= 0.0 && position <= 1.0);

        // Interpolate two color components in a gamma-correct way
        fn gamma_interpolate(a: u8, b: u8, position: f32) -> u8 {
            let gamma = 2.2;
            let a_linear = (a as f32).powf(gamma);
            let b_linear = (b as f32).powf(gamma);
            let lerp = a_linear.lerp(b_linear, position);
            lerp.powf(1.0/gamma).round() as u8
        }

        Color {
            r: gamma_interpolate(self.r, other.r, position),
            g: gamma_interpolate(self.g, other.g, position),
            b: gamma_interpolate(self.b, other.b, position)
        }
    }
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

impl Gradient {
    pub fn new(control_points: Vec<ControlPoint>) -> Gradient {
        let mut points = control_points.clone();
        if points.len() == 0 {
            points.push(ControlPoint::new(128, 128, 128, 0.0));
        }
        if points.len() == 1 {
            let mut cp = points[0];
            cp.position = (cp.position + 0.5).wrap();
            points.push(cp)
        }
        points.sort_by(|a, b| (a.position).partial_cmp(&b.position).unwrap());

        Gradient {
            points: points
        }
    }

    pub fn get(&self, position: f32) -> Color {
        let pos = position.wrap();
        let mut iter = self.iter();
        let mut subgradient = iter.next().unwrap();
        while !subgradient.contains(pos) {
            subgradient = iter.next().unwrap();
        }
        subgradient.color_at(pos)
    }

    pub fn iter(&self) -> GradientIterator {
        GradientIterator {
            index1: self.points.len() - 1,
            gradient: &self
        }
    }
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

#[cfg(test)]
mod tests {
    use super::Color;
    use super::ControlPoint;
    use super::Subgradient;

    #[test]
    fn test_color_lerp() {
        let a = Color::new(255, 0, 0);
        let b = Color::new(186, 186, 0); // gamma-correct midpoint for gamma = 2.2
        let c = Color::new(0, 255, 0);
        assert_eq!(a, a.lerp(c, 0.0));
        assert_eq!(b, a.lerp(c, 0.5));
        assert_eq!(c, a.lerp(c, 1.0));
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
        let color_a = Color::new(60, 0, 0);
        let color_b = Color::new(0, 60, 0);
        let color_c = Color::new(0, 0, 60);
        let a = ControlPoint { color: color_a, position: 0.0 };
        let b = ControlPoint { color: color_b, position: 0.2 };
        let c = ControlPoint { color: color_c, position: 0.7 };

        // Test interval starting at 0.0/1.0
        assert_eq!(a.lerp(b, 0.0), color_a);
        assert_eq!(a.lerp(b, 0.1), color_a.lerp(color_b, 0.5));
        assert_eq!(a.lerp(b, 0.2), color_b);

        // Test middle interval
        assert_eq!(b.lerp(c, 0.2), color_b);
        assert_eq!(b.lerp(c, 0.3), color_b.lerp(color_c, 0.2));
        assert_eq!(b.lerp(c, 0.7), color_c);

        // Test interval ending at 0.0/1.0
        assert_eq!(c.lerp(a, 0.7), color_c);
        assert_eq!(c.lerp(a, 0.8), color_c.lerp(color_a, 1.0/3.0));
        assert_eq!(c.lerp(a, 1.0), color_a);

        // Test interval crossing 0.0/1.0
        assert_eq!(c.lerp(b, 0.7), color_c);
        assert_eq!(c.lerp(b, 0.8), color_c.lerp(color_b, 0.2));
        assert_eq!(c.lerp(b, 0.0), color_c.lerp(color_b, 0.6));
        assert_eq!(c.lerp(b, 0.1), color_c.lerp(color_b, 0.8));
        assert_eq!(c.lerp(b, 0.2), color_b);
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
        let c1 = Color::new(60, 0, 0);
        let c2 = Color::new(0, 60, 0);
        let s = Subgradient::new(
            ControlPoint { color: c1, position: 0.8 },
            ControlPoint { color: c2, position: 0.3 }
        );
        assert_eq!(s.color_at(0.1), c1.lerp(c2, 3.0/5.0));
    }
}
