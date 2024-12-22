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

use std::fs::File;
use std::io::{stdout, Write};
use std::sync::mpsc::channel;
use std::thread::spawn;

mod config;
mod generate;
mod panic;

use config::{parse, setup_logging, Commands, GenerateCommands, UsernameCommands};
use generate::*;
use rand::thread_rng;
use panic::setup_panic;
use lib::{load::*, analyze::analyze, visualize::visualize};

type BoxedError<'a> = Box<dyn std::error::Error + Send + Sync + 'a>;
type UnitResult<'a> = Result<(), BoxedError<'a>>;

fn execute() -> UnitResult<'static> {
    let arguments = parse();

    setup_panic();
    setup_logging(&arguments.verbosity)?;

    match arguments.command {
        Commands::Generate { command } => {
            let (sender, receiver) = channel::<Vec<u8>>();

            let handle = match command {
                GenerateCommands::Bytes {
                    length
                } => spawn(move || create_bytes(sender, length)),
                GenerateCommands::Hex {
                    uppercase,
                    length
                } => spawn(move || create_hex(sender, uppercase, length)),
                GenerateCommands::Base64 {
                    url_safe,
                    length
                } => spawn(move || create_base64(sender, url_safe, length)),
                GenerateCommands::Password {
                    numbers,
                    symbols,
                    length,
                    count
                } => {
                    let character_set = get_character_set(numbers, symbols);

                    spawn(move || create_password(sender, &character_set, length, count))
                },
                GenerateCommands::Passphrase {
                    path,
                    delimiter,
                    separator,
                    length,
                    count
                } => {
                    let mut rng = thread_rng();
                    let wordlist = get_wordlist(path, Some(&delimiter), &mut rng)?;

                    spawn(move || create_passphrase(sender, &wordlist, &separator, length, count))
                },
                GenerateCommands::Username {
                    capitalize,
                    command
                } => match command {
                    UsernameCommands::Simple {
                        length,
                        count
                    } => spawn(move || create_username(sender, capitalize, UsernameKind::Simple, length, count)),
                    UsernameCommands::Complex {
                        length,
                        count
                    } => spawn(move || create_username(sender, capitalize, UsernameKind::Complex, length, count))
                },
                GenerateCommands::Digits {
                    length,
                    count
                } => spawn(move || create_digits(sender, length, count)),
                GenerateCommands::Number {
                    minimum,
                    maximum,
                    count
                } => spawn(move || create_number(sender, minimum, maximum, count)),
                GenerateCommands::Markov {
                    capitalize,
                    path,
                    length_range,
                    model_parameters,
                    cache_control,
                    count
                } => {
                    let (minimum, maximum) = (length_range.minimum, length_range.maximum);
                    let model_parameters = (model_parameters.order, model_parameters.prior, model_parameters.backoff);
                    let cache_control = (cache_control.no_cache, cache_control.rebuild_cache);
                    let generator = get_generator(path, model_parameters, cache_control)?;

                    spawn(move || create_markov(sender, &generator, capitalize, minimum, maximum, count))
                }
            };

            let mut stdout = stdout();

            for message in receiver {
                stdout.write_all(&message)?;
            }

            stdout.flush()?;

            handle.join().unwrap();
        }
        Commands::Analyze { input } => {
            let buffer = read_in(input)?;

            let report = analyze(buffer);

            println!("{report}");
        }
        Commands::Visualize { input, output } => {
            let buffer = read_in(input)?;

            if let Some(path) = output {
                let mut file = File::options()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(path)?;

                visualize(&mut file, &buffer)?;
            }
            else {
                let mut stdout = stdout();

                visualize(&mut stdout, &buffer)?;
            };
        }
    }

    Ok(())
}

fn main() {
    if let Err(error) = execute() {
        panic!("{error}");
    }
}
