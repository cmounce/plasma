use genetics::{Gene, Chromosome};
use fastmath::FastMath;

pub const FORMULA_GENE_SIZE: usize = 4;
pub const NUM_FORMULA_GENES: usize = 3;

// TODO: Figure out how to store precomputed values
struct WaveFormula {
    x_scale: f32,
    y_scale: f32,
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

pub struct PlasmaFormulas {
    wave: WaveFormula,
    rotating_wave: RotatingWaveFormula,
    circular_wave: CircularWaveFormula
}

fn byte_to_float(byte: u8) -> f32 {
    (byte as f32)/255.0
}

fn byte_to_ifloat(byte: u8) -> f32 {
    (byte as f32/255.0*16.0 - 8.0).round()
}

impl WaveFormula {
    fn from_gene(gene: &Gene) -> WaveFormula {
        assert!(gene.data.len() == 4);
        WaveFormula {
            x_scale: byte_to_float(gene.data[0]),
            y_scale: byte_to_float(gene.data[1]),
            scale: byte_to_float(gene.data[2]),
            wave_speed: byte_to_ifloat(gene.data[3])
        }
    }

    #[inline]
    pub fn get_value(&self, x: f32, y: f32, time: f32) -> f32 {
        let x_factor = self.x_scale.cowave();
        let y_factor = self.y_scale.wave();
        (self.scale*(x*x_factor + y*y_factor) + self.wave_speed*time).wave()
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

    #[inline]
    pub fn get_value(&self, x: f32, y: f32, time: f32) -> f32 {
        let x_factor = (self.x_time*time).cowave();
        let y_factor = (self.y_time*time).wave();
        (self.scale*(x*x_factor + y*y_factor) + self.wave_speed*time).wave()
    }
}

impl CircularWaveFormula {
    fn from_gene(gene: &Gene) -> CircularWaveFormula {
        assert!(gene.data.len() == FORMULA_GENE_SIZE);
        CircularWaveFormula {
            x_time: byte_to_ifloat(gene.data[0]),
            y_time: byte_to_ifloat(gene.data[1]),
            scale: byte_to_float(gene.data[2]),
            wave_speed: byte_to_ifloat(gene.data[3])
        }
    }

    #[inline]
    pub fn get_value(&self, x: f32, y: f32, time: f32) -> f32 {
        let dx = x - (self.x_time*time).cowave();
        let dy = y - (self.y_time*time).wave();
        (self.scale*(dx*dx + dy*dy + 0.1).sqrt() + self.wave_speed*time).wave()
    }
}

impl PlasmaFormulas {
    pub fn from_chromosome(c: &Chromosome) -> PlasmaFormulas {
        assert!(c.genes.len() == NUM_FORMULA_GENES);
        PlasmaFormulas {
            wave: WaveFormula::from_gene(&c.genes[0]),
            rotating_wave: RotatingWaveFormula::from_gene(&c.genes[1]),
            circular_wave: CircularWaveFormula::from_gene(&c.genes[2])
        }
    }

    pub fn get_value(&self, x: f32, y: f32, time: f32) -> f32 {
        self.wave.get_value(x, y, time) +
            self.rotating_wave.get_value(x, y, time) +
            self.circular_wave.get_value(x, y, time)
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
