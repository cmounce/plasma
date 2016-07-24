struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

pub struct ColorMapper {
    color1: Color,
    color2: Color
}

impl ColorMapper {
    pub fn new() -> ColorMapper {
        ColorMapper {
            color1: Color {r:0, g:32, b:64},
            color2: Color {r:64, g:96, b:192}
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
