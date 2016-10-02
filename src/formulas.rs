use genetics::{Gene, Chromosome};
use fastmath::FastMath;

pub const FORMULA_GENE_SIZE: usize = 5;
pub const NUM_FORMULA_GENES: usize = 3;

trait Formula {
    fn from_gene(gene: &Gene) -> Self;
    fn set_time(&mut self, time: f32);
    fn get_value(&self, x: f32, y: f32) -> f32;
}

struct WaveFormula {
    amplitude: f32,
    x_scale: f32,
    y_scale: f32,
    scale: f32,
    wave_speed: f32,
    x_factor: f32,
    y_factor: f32,
    wave_position: f32
}

struct RotatingWaveFormula {
    amplitude: f32,
    x_time: f32,
    y_time: f32,
    scale: f32,
    wave_speed: f32,
    x_factor: f32,
    y_factor: f32,
    wave_position: f32
}

struct CircularWaveFormula {
    amplitude: f32,
    x_time: f32,
    y_time: f32,
    scale: f32,
    wave_speed: f32,
    center_x: f32,
    center_y: f32,
    wave_position: f32
}

pub struct PlasmaFormulas {
    wave: WaveFormula,
    rotating_wave: RotatingWaveFormula,
    circular_wave: CircularWaveFormula
}

trait ByteFloat {
    fn to_float(&self) -> f32;
    fn to_ifloat(&self) -> f32;
}

impl ByteFloat for u8 {
    fn to_float(&self) -> f32 {
        (*self as f32)/255.0
    }

    fn to_ifloat(&self) -> f32 {
        (*self as f32/255.0*16.0 - 8.0).round()
    }
}

impl Formula for WaveFormula {
    fn from_gene(gene: &Gene) -> WaveFormula {
        assert!(gene.data.len() == FORMULA_GENE_SIZE);
        WaveFormula {
            amplitude: gene.data[0].to_float(),
            x_scale: gene.data[1].to_float(),
            y_scale: gene.data[2].to_float(),
            scale: gene.data[3].to_float(),
            wave_speed: gene.data[4].to_ifloat(),
            x_factor: 0.0,
            y_factor: 0.0,
            wave_position: 0.0
        }
    }

    fn set_time(&mut self, time: f32) {
        self.x_factor = self.scale*self.x_scale.cowave();
        self.y_factor = self.scale*self.y_scale.wave();
        self.wave_position = self.wave_speed*time;
    }

    #[inline]
    fn get_value(&self, x: f32, y: f32) -> f32 {
        (self.x_factor*x + self.y_factor*y + self.wave_position).wave()*self.amplitude
    }
}

impl Formula for RotatingWaveFormula {
    fn from_gene(gene: &Gene) -> RotatingWaveFormula {
        assert!(gene.data.len() == FORMULA_GENE_SIZE);
        RotatingWaveFormula {
            amplitude: gene.data[0].to_float(),
            x_time: gene.data[1].to_ifloat(),
            y_time: gene.data[2].to_ifloat(),
            scale: gene.data[3].to_float(),
            wave_speed: gene.data[4].to_ifloat(),
            x_factor: 0.0,
            y_factor: 0.0,
            wave_position: 0.0
        }
    }

    fn set_time(&mut self, time: f32) {
        self.x_factor = self.scale*(self.x_time*time).cowave();
        self.y_factor = self.scale*(self.y_time*time).wave();
        self.wave_position = self.wave_speed*time;
    }

    #[inline]
    fn get_value(&self, x: f32, y: f32) -> f32 {
        (self.x_factor*x + self.y_factor*y + self.wave_position).wave()*self.amplitude
    }
}

impl Formula for CircularWaveFormula {
    fn from_gene(gene: &Gene) -> CircularWaveFormula {
        assert!(gene.data.len() == FORMULA_GENE_SIZE);
        CircularWaveFormula {
            amplitude: gene.data[0].to_float(),
            x_time: gene.data[1].to_ifloat(),
            y_time: gene.data[2].to_ifloat(),
            scale: gene.data[3].to_float(),
            wave_speed: gene.data[4].to_ifloat(),
            center_x: 0.0,
            center_y: 0.0,
            wave_position: 0.0
        }
    }

    fn set_time(&mut self, time: f32) {
        self.center_x = (self.x_time*time).cowave();
        self.center_y = (self.y_time*time).wave();
        self.wave_position = self.wave_speed*time;
    }

    #[inline]
    fn get_value(&self, x: f32, y: f32) -> f32 {
        let dx = x - self.center_x;
        let dy = y - self.center_y;
        (self.scale*(dx*dx + dy*dy + 0.1).sqrt() + self.wave_position).wave()*self.amplitude
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

    pub fn set_time(&mut self, time: f32) {
        self.wave.set_time(time);
        self.rotating_wave.set_time(time);
        self.circular_wave.set_time(time);
    }

    pub fn get_value(&self, x: f32, y: f32) -> f32 {
        self.wave.get_value(x, y) +
            self.rotating_wave.get_value(x, y) +
            self.circular_wave.get_value(x, y)
    }
}

#[cfg(test)]
mod tests {
    use fastmath::FastMath;
    use genetics::Gene;
    use super::FORMULA_GENE_SIZE;
    use super::{Formula,WaveFormula,RotatingWaveFormula,CircularWaveFormula};

    // Compares a Formula with a reference implementation at various coordinates and times.
    // - optimized is the Formula to test.
    // - reference is the reference implementation that maps (x, y, time) to a f32 value.
    fn test_formula<F: Formula, C>(formula: &mut F, reference: C)
        where C : Fn(f32, f32, f32) -> f32 {
        // Helper for loops below
        fn range(low: f32, high: f32, step: f32) -> Vec<f32> {
            assert!(low < high && step > 0.0);
            let mut result = vec![];
            let mut x = low;
            while x < high {
                result.push(x);
                x += step;
            }
            result
        }
        for x in range(-2.0, 2.0, 0.1) {
            for y in range(-2.0, 2.0, 0.1) {
                for time in range(0.0, 2.0, 0.1) {
                    // Verify that formula matches its reference implementation
                    formula.set_time(time);
                    let reference_value = reference(x, y, time);
                    let formula_value = formula.get_value(x, y);
                    assert!((reference_value - formula_value).abs() < 0.001);

                    // Verify that formula(time) equals formula(time + 1.0)
                    formula.set_time(time + 1.0);
                    let next_formula_value = formula.get_value(x, y);
                    assert!((formula_value - next_formula_value).abs() < 0.001);
                }
            }
        }
    }

    #[test]
    fn test_wave_get_value() {
        let g = Gene::rand(FORMULA_GENE_SIZE);
        let mut f = WaveFormula::from_gene(&g);

        let x_factor = f.x_scale.cowave();
        let y_factor = f.y_scale.wave();
        let scale = f.scale;
        let wave_speed = f.wave_speed;
        let amplitude = f.amplitude;
        test_formula(&mut f, |x, y, time| {
            (scale*(x*x_factor + y*y_factor) + wave_speed*time).wave()*amplitude
        });
    }

    #[test]
    fn test_rotating_wave_get_value() {
        let g = Gene::rand(FORMULA_GENE_SIZE);
        let mut f = RotatingWaveFormula::from_gene(&g);

        let x_time = f.x_time;
        let y_time = f.y_time;
        let scale = f.scale;
        let wave_speed = f.wave_speed;
        let amplitude = f.amplitude;
        test_formula(&mut f, |x, y, time| {
            let x_factor = (x_time*time).cowave();
            let y_factor = (y_time*time).wave();
            (scale*(x*x_factor + y*y_factor) + wave_speed*time).wave()*amplitude
        });
    }

    #[test]
    fn test_circular_wave_get_value() {
        let g = Gene::rand(FORMULA_GENE_SIZE);
        let mut f = CircularWaveFormula::from_gene(&g);

        let x_time = f.x_time;
        let y_time = f.y_time;
        let scale = f.scale;
        let wave_speed = f.wave_speed;
        let amplitude = f.amplitude;
        test_formula(&mut f, |x, y, time| {
            let dx = x - (x_time*time).cowave();
            let dy = y - (y_time*time).wave();
            (scale*(dx*dx + dy*dy + 0.1).sqrt() + wave_speed*time).wave()*amplitude
        });
    }
}
