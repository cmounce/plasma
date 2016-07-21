pub struct ColorMapper {
}

impl ColorMapper {
    pub fn new() -> ColorMapper {
        ColorMapper {
        }
    }

    pub fn convert(&self, value: f32) -> (u8, u8, u8) {
        let value_adj = value - value.floor();
        let brightness = (value_adj - 0.5).abs()*2.0;
        let byte = (brightness*255.0).round() as u8;
        (byte/4, byte/4 + 32, byte/2 + 64)
    }
}
