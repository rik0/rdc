extern crate bigdecimal;
extern crate num;
extern crate num_bigint;

use std::io::Write;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use std::fmt;

mod instructions;
mod parse;
#[macro_use]
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

    {
        // TODO probably it is a bit better to copy some stuff in the stack that should
        // own all the data that gets in...
        let mut stdout = std::io::stdout();
        let mut stderr = std::io::stderr();
        let mut vm = vm::VM::new(&mut stdout, &mut stderr);
        for program_source in program_sources {
            let mut source_code = Vec::new();
            match program_source.into_bytes(&mut source_code) {
                Ok(bytes) => match parse::parse(&source_code[..bytes]) {
                    Err(parse_error) => eprintln!("parse error {}", parse_error),
                    Ok(instructions) => {
                        if let Err(ioerror) = vm.eval(&instructions[..]) {
                            eprintln!("ioerror: {}", ioerror)
                        }
                    }
                },
                Err(error) => {
                    eprintln!("error processing file {}", error);
                }
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
