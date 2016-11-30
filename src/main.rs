extern crate getopts;
extern crate gif;
extern crate sdl2;

mod colormapper;
mod fastmath;
mod file;
mod formulas;
mod genetics;
mod gradient;
mod interactive;
mod renderer;
mod settings;

use colormapper::{NUM_COLOR_GENES, CONTROL_POINT_GENE_SIZE};
use formulas::{NUM_FORMULA_GENES, FORMULA_GENE_SIZE};
use getopts::{Options, Matches};
use genetics::{Chromosome, Genome, Population};
use settings::{GeneticSettings,RenderingSettings,OutputMode,OutputSettings,PlasmaSettings};
use std::cmp::max;
use std::env;
use std::io::Write;

const STARTING_POPULATION_SIZE: usize = 8;
const MAX_POPULATION_SIZE: usize = 32;

fn main() {
    let opts = create_options();
    let matches = match opts.parse(env::args()) {
        Ok(m) => m,
        Err(s) => {
            writeln!(&mut std::io::stderr(), "Error parsing arguments: {:?}", s).unwrap();
            return;
        }
    };
    let params = match build_plasma_settings(matches) {
        Ok(params) => params,
        Err(s) => {
            writeln!(&mut std::io::stderr(), "{}", s).unwrap();
            return;
        }
    };

    match params.output.mode {
        OutputMode::File{..} => file::output_gif(params),
        OutputMode::Interactive => interactive::run_interactive(params)
    };
}

fn create_options() -> Options {
    let mut opts = Options::new();
    opts.optflag("d", "dithering", "Force dithering");
    opts.optopt("p", "palette", "Render using a color palette of a given size", "N");
    opts.optopt("f", "fps", "Frames per second", "N");
    opts.optopt("l", "loop-duration", "Seconds until the animation loops", "N");
    opts.optopt("o", "output", "Output to a file (GIF) instead of to a window", "FILE");
    opts.optflag("v", "verbose", "Print stats while running");
    opts.optopt("w", "width", "Width, in pixels", "X");
    opts.optopt("h", "height", "Height, in pixels", "Y");
    opts.optflag("", "help", "Show this help text");
    opts
}

fn build_plasma_settings(matches: Matches) -> Result<PlasmaSettings, String> {
    // Read genomes from free arguments
    let genome_strings = &matches.free[1..];
    let mut genomes = vec![];
    for genome_string in genome_strings {
        match Genome::from_base64(genome_string) {
            Ok(g) => genomes.push(g),
            Err(..) => return Err(format!("Couldn't parse {}", genome_string))
        };
    }

    // Set up genetic settings
    if genomes.len() == 0 {
        for _ in 0..STARTING_POPULATION_SIZE {
            genomes.push(Genome {
                pattern: Chromosome::rand(NUM_FORMULA_GENES, FORMULA_GENE_SIZE),
                color: Chromosome::rand(NUM_COLOR_GENES, CONTROL_POINT_GENE_SIZE)
            });
        }
    }
    let starting_genome = genomes[0].clone();
    let mut population = Population::new(max(MAX_POPULATION_SIZE, genomes.len()));
    for genome in genomes {
        population.add(genome);
    }
    let genetic_settings = GeneticSettings {
        genome: starting_genome,
        population: population
    };

    // Set up output settings
    let output_mode = if matches.opt_present("o") {
        OutputMode::File { path: matches.opt_str("o").unwrap() }
    } else {
        OutputMode::Interactive
    };
    let output_settings = OutputSettings {
        mode: output_mode,
        verbose: matches.opt_present("v")
    };

    // Set up rendering settings
    let mut rendering_settings = match output_settings.mode {
        OutputMode::Interactive => RenderingSettings {
            dithering: false,
            frames_per_second: 15.0,
            loop_duration: 60.0,
            palette_size: None,
            width: 640,
            height: 480
        },
        OutputMode::File{..} => RenderingSettings {
            dithering: true,
            frames_per_second: 10.0,
            loop_duration: 60.0,
            palette_size: Some(64),
            width: 320,
            height: 240
        }
    };
    if matches.opt_present("d") {
        rendering_settings.dithering = true;
        if rendering_settings.palette_size.is_none() {
            rendering_settings.palette_size = Some(255);
        }
    }
    if matches.opt_present("f") {
        let fps_str = matches.opt_str("f").unwrap();
        rendering_settings.frames_per_second = match fps_str.parse() {
            Ok(f) if f > 0.0 => f,
            _ => return Err(format!("Not a positive number: {}", fps_str))
        };
    }
    if matches.opt_present("l") {
        let loop_duration_str = matches.opt_str("l").unwrap();
        rendering_settings.loop_duration = match loop_duration_str.parse() {
            Ok(n) if n > 0.0 => n,
            _ => return Err(format!("Not a positive number: {}", loop_duration_str))
        };
    }
    if matches.opt_present("p") {
        let palette_size_str = matches.opt_str("p").unwrap();
        rendering_settings.palette_size = match palette_size_str.parse() {
            Ok(n) if 2 <= n && n <= 255 => Some(n),
            _ => return Err(format!("Not an integer from 2 to 255: {}", palette_size_str))
        };
    }
    if matches.opt_present("w") || matches.opt_present("h") {
        if !matches.opt_present("w") || !matches.opt_present("h") {
            return Err("Width and height must both be specified".to_string());
        }
        let width_str = matches.opt_str("w").unwrap();
        rendering_settings.width = match width_str.parse() {
            Ok(w) if w > 0 => w,
            _ => return Err(format!("Not a positive integer: {}", width_str))
        };
        let height_str = matches.opt_str("h").unwrap();
        rendering_settings.height = match height_str.parse() {
            Ok(h) if h > 0 => h,
            _ => return Err(format!("Not a positive integer: {}", height_str))
        };
    }

    Ok(PlasmaSettings {
        genetics: genetic_settings,
        rendering: rendering_settings,
        output: output_settings
    })
}
