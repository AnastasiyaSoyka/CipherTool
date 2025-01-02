extern crate lib;
extern crate rand;
extern crate log;
extern crate clap;
extern crate zstd;
extern crate digest;
extern crate sha1;
extern crate sha2;
extern crate hex;
extern crate rayon;

mod config;
mod delegate;
mod panic;
mod command;

use config::{parse, setup_logging, Commands};
use panic::setup_panic;
use command::{handle_create_command, handle_analyze_command, handle_visualize_command};

type BoxedError<'a> = Box<dyn std::error::Error + Send + Sync + 'a>;
type UnitResult<'a> = Result<(), BoxedError<'a>>;

fn execute() -> UnitResult<'static> {
    let arguments = parse();

    setup_panic();
    setup_logging(&arguments.verbosity)?;

    match arguments.command {
        Commands::Create { command } => handle_create_command(command)?,
        Commands::Analyze { input } => handle_analyze_command(input)?,
        Commands::Visualize { input, output } => handle_visualize_command(input, output)?
    };

    Ok(())
}

fn main() {
    if let Err(error) = execute() {
        panic!("{error}");
    }
}
