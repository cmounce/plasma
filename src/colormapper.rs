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
}

// impl Ord for ControlPoint {
//     fn cmp(&self, other: &Self) -> Ordering {
//         match self.position.partial_cmp(other.position) {
//             Some(ordering) => ordering,
//             None => panic!("Tried to compare {} with {}", self, other)
//         }
//     }
// }

struct Subgradient<'a> {
    index1: usize,
    index2: usize,
    gradient: &'a Gradient
}

impl<'a> Subgradient<'a> {
    fn get_point1(&self) -> &ControlPoint {
        &self.gradient.points[self.index1]
    }

    fn get_point2(&self) -> &ControlPoint {
        &self.gradient.points[self.index2]
    }

    fn start(&self) -> f32 {
        self.get_point1().position
    }

    fn end(&self) -> f32 {
        self.get_point2().position
    }

    fn len(&self) -> f32 {
        let distance = self.end() - self.start();
        distance - distance.floor() // if distance is negative, return 1 - distance
    }

    fn color_at(&self, position: f32) -> Color {
        let length = self.len();
        assert!(length != 0.0);
        let adj_position = (position - self.get_point1().position)/length;

        self.get_point1().color.lerp(self.get_point2().color, adj_position)
    }

    fn next(&self) -> Subgradient {
        Subgradient {
            index1: self.index2,
            index2: (self.index2 + 1) % self.gradient.points.len(),
            gradient: self.gradient
        }
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

    fn color_at(&self, position: f32) -> Color {
        self.points[0].color
        // let result = self.points.binary_search_by_key(&position, |&point| &point.position);
        // if let Ok(index) = result {
        //     self.points[index].color
        // } else {
        //     let Err(index2) = result;
        //     let index1 = (index2 - 1) % self.points.len();
        //
        //     let subgradient = Subgradient {
        //         index1: index1,
        //         index2: index2,
        //         gradient: &self
        //     };
        //     subgradient.color_at(position)
        // }
    }

    fn first(&self) -> Subgradient {
        Subgradient {
            index1: self.points.len() - 1,
            index2: 0,
            gradient: &self
        }
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
        for i in 0..LOOKUP_TABLE_SIZE {
            let position = (i as f32)/(LOOKUP_TABLE_SIZE as f32);
            self.lookup_table[i] = gradient.color_at(position);
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
