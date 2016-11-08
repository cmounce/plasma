extern crate getopts;
extern crate gif;
extern crate sdl2;

mod colormapper;
mod fastmath;
mod formulas;
mod genetics;
mod gradient;
mod interactive;
mod renderer;

use colormapper::{NUM_COLOR_GENES, CONTROL_POINT_GENE_SIZE};
use formulas::{NUM_FORMULA_GENES, FORMULA_GENE_SIZE};
use getopts::Options;
use genetics::{Chromosome, Genome, Population};
use interactive::InteractiveParameters;
use std::cmp::max;
use std::env;
use std::io::Write;
use std::process::exit;

const STARTING_POPULATION_SIZE: usize = 8;
const MAX_POPULATION_SIZE: usize = 32;

fn rand_genome() -> Genome {
    Genome {
        pattern: Chromosome::rand(NUM_FORMULA_GENES, FORMULA_GENE_SIZE),
        color: Chromosome::rand(NUM_COLOR_GENES, CONTROL_POINT_GENE_SIZE)
    }
}

fn read_genome(data: &str) -> Genome {
    if let Ok(genome) = Genome::from_base64(data) {
        genome
    } else {
        writeln!(&mut std::io::stderr(), "Invalid genome string: {}", data).unwrap();
        exit(1);
    }
}

fn main() {
    let mut opts = Options::new();
    opts.optflag("v", "verbose", "Print stats while running");
    let matches = match opts.parse(env::args()) {
        Ok(m) => m,
        Err(_) => {
            writeln!(&mut std::io::stderr(), "Bad arguments").unwrap();
            return;
        }
    };

    // Deserialize starting genome,
    let genome_strings = &matches.free[1..];
    let starting_genome = if genome_strings.len() == 0 {
        rand_genome()
    } else {
        read_genome(&genome_strings[0])
    };
    let mut population = Population::new(max(genome_strings.len(), MAX_POPULATION_SIZE));
    if genome_strings.len() == 0 {
        for _ in 0..STARTING_POPULATION_SIZE {
            population.add(rand_genome());
        }
    } else {
        for genome_string in genome_strings {
            population.add(read_genome(genome_string));
        }
    }
    interactive::run_interactive(InteractiveParameters {
        print_stats: matches.opt_present("v"),
        starting_genome: starting_genome,
        population: population
    });
}
