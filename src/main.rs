extern crate num;

use std::io::Write;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use std::fmt;

mod instructions;
mod parse;
mod dcstack;
mod vm;


enum ProgramSource {
    Text(String),
    File(String),
}

impl ProgramSource {
    fn into_bytes<'a>(self, buffer: &mut Vec<u8>) -> Result<usize, std::io::Error> {
        match self {
            ProgramSource::Text(text_str) => {
                return Ok(buffer.write(text_str.as_bytes())?);
            }
            ProgramSource::File(filename) => {
                let path = Path::new(&filename);
                let mut file = File::open(path)?;
                return Ok(file.read_to_end(buffer)?);
            }
        }
    }
}

fn main() {
    // let us implement the real app to understand approaches to ownership

    let mut vm = vm::VM::<u64>::new();

    let mut args = std::env::args().skip(1);

    let mut program_sources = Vec::new();
    let mut positional_program_sources = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_ref() {
            "-e" | "--expression" => match args.next() {
                Some(text) => program_sources.push(ProgramSource::Text(text)),
                None => print_help(1),
            },
            "-f" | "--file" => match args.next() {
                Some(file) => program_sources.push(ProgramSource::File(file)),
                None => print_help(1),
            },
            "-h" | "--help" => print_help(0),
            "-v" | "--version" => print_version(0),
            _ => {
                positional_program_sources.push(ProgramSource::File(arg.to_string()));
            }
        }
    }

    for program_source in program_sources {
        let mut source_code = Vec::new();
        match program_source.into_bytes(&mut source_code) {
            Ok(bytes) => match parse::parse(&source_code[..bytes]) {
                Err(parse_error) => {
                    // TODO I should use a description
                    eprintln!("parse error {}", parse_error);
                }
                Ok(instructions) => {
                    if let Some(error) = vm.eval(&instructions[..]).err() {
                        eprintln!("error processing {}: {}", 
                            String::from_utf8(source_code.clone()) // TODO: fixme
                                .unwrap_or("program is not utf8".to_string()), 
                            error);
                    }
                }
            },
            Err(error) => {
                eprintln!("error processing file {}", error);
            }
        }
    }
}

fn print_help(code: i32) {
    std::process::exit(code);
}

fn print_version(code: i32) {
    std::process::exit(code);
}