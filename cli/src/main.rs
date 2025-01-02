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
mod delegate;
mod panic;

use config::{parse, setup_logging, Commands, CreateCommands, TimestampCommands, UsernameCommands};
use delegate::{create_serial, create_parallel};
use rand::thread_rng;
use panic::setup_panic;
use lib::{load::*, generators::*, analyze::analyze, visualize::visualize, time::*};

type BoxedError<'a> = Box<dyn std::error::Error + Send + Sync + 'a>;
type UnitResult<'a> = Result<(), BoxedError<'a>>;

const LINE_FEED: [u8; 1] = [b'\n'];

fn execute() -> UnitResult<'static> {
    let arguments = parse();

    setup_panic();
    setup_logging(&arguments.verbosity)?;

    match arguments.command {
        Commands::Create { command } => {
            let (sender, receiver) = channel::<Vec<u8>>();
            let total: usize;

            let handle = match command {
                CreateCommands::Timestamp { command, format } => match command {
                    TimestampCommands::Utc => {
                        total = 1;

                        spawn(move || create_serial(sender, || create_timestamp_utc(format)))
                    },
                    TimestampCommands::Local => {
                        total = 1;

                        spawn(move || create_serial(sender, || create_timestamp_local(format)))
                    }
                },
                CreateCommands::Bytes { length } => {
                    total = 1;

                    spawn(move || create_serial(sender, || generate_bytes(length)))
                },
                CreateCommands::Hex { uppercase, length } => {
                    total = 1;

                    spawn(move || create_serial(sender, || generate_hex(uppercase, length)))
                },
                CreateCommands::Base64 { url_safe, length } => {
                    total = 1;

                    spawn(move || create_serial(sender, || generate_base64(url_safe, length)))
                },
                CreateCommands::Password { numbers, symbols, length, count } => {
                    total = count.unwrap_or(1);

                    let character_set = get_character_set(numbers, symbols);

                    spawn(move || create_parallel(sender, count, || generate_password(&character_set, length)))
                },
                CreateCommands::Passphrase { path, delimiter, separator, length, count } => {
                    total = count.unwrap_or(1);

                    let mut rng = thread_rng();
                    let wordlist = get_wordlist(path, Some(&delimiter), &mut rng)?;

                    spawn(move || create_parallel(sender, count, || generate_passphrase(&wordlist, &separator, length)))
                },
                CreateCommands::Username { capitalize, command } => match command {
                    UsernameCommands::Simple { length, count } => {
                        total = count.unwrap_or(1);

                        spawn(move || create_parallel(sender, count, || generate_simple_username(capitalize, length)))
                    },
                    UsernameCommands::Complex { length, count } => {
                        total = count.unwrap_or(1);

                        spawn(move || create_parallel(sender, count, || generate_complex_username(capitalize, length)))
                    }
                },
                CreateCommands::Digits { length, count } => {
                    total = count.unwrap_or(1);

                    spawn(move || create_parallel(sender, count, || generate_digits(length)))
                },
                CreateCommands::Number { minimum, maximum, count } => {
                    total = count.unwrap_or(1);

                    spawn(move || create_parallel(sender, count, || generate_number(minimum, maximum)))
                },
                CreateCommands::Markov {
                    capitalize,
                    path,
                    length_range,
                    model_parameters,
                    cache_control,
                    count
                } => {
                    total = count.unwrap_or(1);

                    let (minimum, maximum) = (length_range.minimum, length_range.maximum);
                    let model_parameters = (model_parameters.order, model_parameters.prior, model_parameters.backoff);
                    let cache_control = (cache_control.no_cache, cache_control.rebuild_cache);
                    let generator = get_generator(path, model_parameters, cache_control)?;

                    spawn(move || create_parallel(sender, count, || generate_markov(
                        &generator,
                        capitalize,
                        minimum,
                        maximum,
                        &mut thread_rng()
                    )))
                }
            };

            let mut stdout = stdout();
            let mut counter = 0;

            for message in receiver {
                counter += 1;

                if counter != total {
                    stdout.write_all(&message)?;
                    stdout.write_all(&LINE_FEED)?;
                }
                else {
                    stdout.write_all(&message)?;
                }
            }

            stdout.flush()?;

            handle.join().unwrap();
        },
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
