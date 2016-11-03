extern crate sdl2;
extern crate getopts;

mod colormapper;
mod fastmath;
mod formulas;
mod genetics;
mod gradient;
mod interactive;
mod renderer;

use getopts::Options;
use interactive::InteractiveParameters;
use std::env;
use std::io::Write;

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
    interactive::run_interactive(InteractiveParameters {
        print_stats: matches.opt_present("v")
    });
}
