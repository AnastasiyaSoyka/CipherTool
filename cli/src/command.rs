use std::fs::File;
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::thread::spawn;

use crate::*;

use config::{CreateCommands, TimestampCommands, UsernameCommands};
use delegate::{create_serial, create_parallel};
use rand::thread_rng;
use lib::{load::*, generators::*, analyze::analyze, visualize::visualize, time::*};

type BoxedError<'a> = Box<dyn std::error::Error + Send + Sync + 'a>;
type UnitResult<'a> = Result<(), BoxedError<'a>>;

/**
 * The size of the buffer used to write to stdout.
 */
const STDOUT_BUFFER_SIZE: usize = 65536;

/**
 * Perform the write operation to stdout.
 */
fn write_to_stdout(receiver: Receiver<Vec<u8>>, total: usize) -> UnitResult<'static> {
    let mut stdout = stdout();
    let mut counter = 0;
    let mut buffer: [u8; STDOUT_BUFFER_SIZE] = [0; STDOUT_BUFFER_SIZE];
    let mut size: usize = 0;

    for mut message in receiver {
        counter += 1;

        while message.len() > 0 {
            // The number of bytes which should be copied to the buffer.
            let length = message.len().min(STDOUT_BUFFER_SIZE);

            // If the buffer doesn't have enough space left, write the buffer to stdout and reset.
            if (size + length) + 1 >= STDOUT_BUFFER_SIZE {
                stdout.write_all(&buffer)?;
                size = 0;
            }

            // Copy the message to the buffer.
            buffer[size..size + length].copy_from_slice(message.drain(..length).as_slice());

            // Add the number of bytes copied to the buffer to the buffer size.
            size += length;
        }

        // Write a line feed to the buffer if there are more messages to write.
        if counter != total {
            buffer[size] = b'\n';
            size += 1;
        }
    }

    // Write any remaining data in the buffer.
    if size != 0 {
        stdout.write_all(&buffer[..size])?;
    }

    stdout.flush()?;

    Ok(())
}

pub fn handle_create_command(command: CreateCommands) -> UnitResult<'static> {
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

    write_to_stdout(receiver, total)?;

    handle.join().unwrap();

    Ok(())
}

pub fn handle_analyze_command(input: Option<PathBuf>) -> UnitResult<'static> {
    let buffer = read_in(input)?;
    let report = analyze(buffer);

    println!("{report}");

    Ok(())
}

pub fn handle_visualize_command(input: Option<PathBuf>, output: Option<PathBuf>) -> UnitResult<'static> {
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

    Ok(())
}
