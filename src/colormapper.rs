use std::cmp::Ordering;

const LOOKUP_TABLE_SIZE: usize = 256;

#[derive(Copy,Clone)]
struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

impl Color {
    fn lerp(&self, other: Color, position: f32) -> Color {
        assert!(position >= 0.0 && position <= 1.0);
        let opposite = 1.0 - position;
        Color {
            r: ((self.r as f32)*position + (other.r as f32)*opposite).round() as u8,
            g: ((self.g as f32)*position + (other.g as f32)*opposite).round() as u8,
            b: ((self.b as f32)*position + (other.b as f32)*opposite).round() as u8
        }
    }
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
            position: position - position.floor()
        }
    }

    fn lerp(&self, other: ControlPoint, position: f32) -> Color {
        let distance = (other.position - self.position) % 1.0;
        assert!(distance > 0.0);
        let adj_position = (position*distance + self.position) % 1.0;
        self.color.lerp(other.color, adj_position)
        // TODO: Test this function
    }
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
