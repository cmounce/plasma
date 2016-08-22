use fastmath::FastMath;
use genetics::Gene;

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

    fn from_hsv(hue: f32, saturation: f32, value: f32) -> Color {
        let h = hue.wrap();
        let s = saturation.clamp(0.0, 1.0);
        let v = value.clamp(0.0, 1.0);

        let sector = (h*6.0).floor() % 6.0;
        let offset = (h*6.0).fract();
        let upper = v;
        let lower = v*(1.0 - s);
        let (r, g, b) = match sector {
            0.0 => (upper, lower.lerp(upper, offset), lower),
            1.0 => (upper.lerp(lower, offset), upper, lower),
            2.0 => (lower, upper, lower.lerp(upper, offset)),
            3.0 => (lower, upper.lerp(lower, offset), upper),
            4.0 => (lower.lerp(upper, offset), lower, upper),
            5.0 => (upper, lower, upper.lerp(lower, offset)),
            _ => panic!("Invalid sector value {}", sector)
        };

        Color::new(
            (r*255.0).round() as u8,
            (g*255.0).round() as u8,
            (b*255.0).round() as u8
        )
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

#[test]
fn test_color_from_hsv() {
    /*
     * H: red -> green -> blue
     * S: white -> color
     * V: black -> color/white
     */
    // Test value = 0
    assert_eq!(Color::from_hsv(0.0, 0.0, 0.0), Color::new(0, 0, 0));
    assert_eq!(Color::from_hsv(0.0, 1.0, 0.0), Color::new(0, 0, 0));
    assert_eq!(Color::from_hsv(0.5, 0.0, 0.0), Color::new(0, 0, 0));
    assert_eq!(Color::from_hsv(0.5, 1.0, 0.0), Color::new(0, 0, 0));

    // Test primaries and secondaries
    assert_eq!(Color::from_hsv(0.0,     1.0, 1.0), Color::new(255, 0,   0));
    assert_eq!(Color::from_hsv(1.0/6.0, 1.0, 1.0), Color::new(255, 255, 0));
    assert_eq!(Color::from_hsv(2.0/6.0, 1.0, 1.0), Color::new(0,   255, 0));
    assert_eq!(Color::from_hsv(3.0/6.0, 1.0, 1.0), Color::new(0,   255, 255));
    assert_eq!(Color::from_hsv(4.0/6.0, 1.0, 1.0), Color::new(0,   0,   255));
    assert_eq!(Color::from_hsv(5.0/6.0, 1.0, 1.0), Color::new(255, 0,   255));
    assert_eq!(Color::from_hsv(1.0,     1.0, 1.0), Color::new(255, 0,   0));

    // Test saturation
    assert_eq!(Color::from_hsv(0.0, 1.0, 1.0),      Color::new(255, 0, 0));
    assert_eq!(Color::from_hsv(0.0, 2.0/3.0, 1.0),  Color::new(255, 85, 85));
    assert_eq!(Color::from_hsv(0.0, 1.0/3.0, 1.0),  Color::new(255, 170, 170));
    // TODO: more tests?
}


#[derive(Copy,Clone,Debug)]
struct ControlPoint {
    color: Color,
    position: f32
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

    // TODO: write tests
    fn from_gene(gene: &Gene) -> Option<ControlPoint> {
        assert!(gene.data.len() == 5);
        let activation_threshold = 160;
        if gene.data[0] > activation_threshold {
            let h = (gene.data[1] as f32)/256.0; // allow h = 1.0
            let s = (gene.data[2] as f32)/256.0; // allow s = 1.0
            let v = (gene.data[3] as f32)/256.0; // allow v = 1.0
            let position = (gene.data[4] as f32)/255.0; // disallow position = 1.0
            Some(ControlPoint {
                color: Color::from_hsv(h, s, v),
                position: position
            })
        } else {
            None
        }
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
        if self.point1.position <= self.point2.position {
            self.point1.position <= adj_position && adj_position <= self.point2.position
        } else {
            adj_position <= self.point2.position || self.point1.position <= adj_position
        }
    }

    fn color_at(&self, position: f32) -> Color {
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


struct Gradient {
    points: Vec<ControlPoint>
}

impl Gradient {
    fn new(control_points: Vec<ControlPoint>) -> Gradient {
        assert!(control_points.len() >= 2);
        let mut points = control_points.clone();
        points.sort_by(|a, b| (a.position).partial_cmp(&b.position).unwrap());

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


pub struct ColorMapper {
    lookup_table: [Color; LOOKUP_TABLE_SIZE]
}

impl ColorMapper {
    pub fn new() -> ColorMapper {
        let mut lookup_table = [Color {r:0, g:0, b:0}; LOOKUP_TABLE_SIZE];
        let gradient = Gradient::new(vec![
            ControlPoint::new(0, 32, 64, 0.0),
            ControlPoint::new(64, 96, 192, 0.5)
        ]);
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

    pub fn convert(&self, value: f32) -> (u8, u8, u8) {
        let index = (value.wrap()*(LOOKUP_TABLE_SIZE as f32)).floor() as usize % LOOKUP_TABLE_SIZE;
        let color = self.lookup_table[index];
        (color.r, color.g, color.b)
    }
}
