use genetics::{Genome, Population};

pub struct PlasmaSettings {
    pub genetics: GeneticSettings,
    pub rendering: RenderingSettings,
    pub output: OutputSettings
}

pub struct GeneticSettings {
    pub genome: Genome,
    pub population: Population
}

#[derive(Clone,Debug)]
pub struct RenderingSettings {
    pub dithering: bool,
    pub frames_per_second: f32,
    pub loop_duration: f32,
    pub palette_size: Option<usize>,
    pub width: usize,
    pub height: usize
}

#[derive(Debug)]
pub struct OutputSettings {
    pub mode: OutputMode,
    pub verbose: bool
}

#[derive(Debug)]
pub enum OutputMode {
    File {path: String},
    Interactive
}
