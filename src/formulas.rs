use genetics::Gene;

// TODO: Figure out how to store precomputed values
// TODO: Write inline fns for calculating plasma at (x, y)
struct WaveFormula {
    x_stretch: f32,
    y_stretch: f32,
    scale: f32,
    wave_speed: f32
}

struct RotatingWaveFormula {
    x_time: f32,
    y_time: f32,
    scale: f32,
    wave_speed: f32
}

struct CircularWaveFormula {
    x_time: f32,
    y_time: f32,
    scale: f32,
    wave_speed: f32
}

struct PlasmaFormulas {
    wave: WaveFormula,
    rotating_wave: RotatingWaveFormula,
    circular_wave: CircularWaveFormula
}

fn byte_to_float(byte: u8) -> f32 {
    (byte as f32)/64.0
}

fn byte_to_ifloat(byte: u8) -> f32 {
    (byte as f32/255.0*16.0 - 8.0).round()
}

impl WaveFormula {
    fn from_gene(gene: &Gene) -> WaveFormula {
        assert!(gene.data.len() == 4);
        WaveFormula {
            x_stretch: byte_to_float(gene.data[0]),
            y_stretch: byte_to_float(gene.data[1]),
            scale: byte_to_float(gene.data[2]),
            wave_speed: byte_to_ifloat(gene.data[3])
        }
    }
}

impl RotatingWaveFormula {
    fn from_gene(gene: &Gene) -> RotatingWaveFormula {
        assert!(gene.data.len() == 4);
        RotatingWaveFormula {
            x_time: byte_to_ifloat(gene.data[0]),
            y_time: byte_to_ifloat(gene.data[1]),
            scale: byte_to_float(gene.data[2]),
            wave_speed: byte_to_ifloat(gene.data[3])
        }
    }
}

impl CircularWaveFormula {
    fn from_gene(gene: &Gene) -> CircularWaveFormula {
        assert!(gene.data.len() == 4);
        CircularWaveFormula {
            x_time: byte_to_ifloat(gene.data[0]),
            y_time: byte_to_ifloat(gene.data[1]),
            scale: byte_to_float(gene.data[2]),
            wave_speed: byte_to_ifloat(gene.data[3])
        }
    }
}

#[cfg(test)]
mod tests {
    use genetics::Gene;
    use super::WaveFormula;
    use super::RotatingWaveFormula;
    use super::CircularWaveFormula;

    #[test]
    fn test_wave_formula_from_gene() {
        let g = Gene::rand(4);
        let wf = WaveFormula::from_gene(&g);
        assert!(wf.wave_speed.fract() == 0.0);
    }

    #[test]
    fn test_rotating_wave_formula_from_gene() {
        let g = Gene::rand(4);
        let wf = RotatingWaveFormula::from_gene(&g);
        assert!(wf.x_time.fract() == 0.0);
        assert!(wf.y_time.fract() == 0.0);
        assert!(wf.wave_speed.fract() == 0.0);
    }

    #[test]
    fn test_circular_wave_formula_from_gene() {
        let g = Gene::rand(4);
        let cf = CircularWaveFormula::from_gene(&g);
        assert!(cf.x_time.fract() == 0.0);
        assert!(cf.y_time.fract() == 0.0);
        assert!(cf.wave_speed.fract() == 0.0);
    }
}
