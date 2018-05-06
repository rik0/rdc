extern crate bigdecimal;
extern crate num;
extern crate num_bigint;

mod instructions;
mod parse;
#[macro_use]
mod dcstack;
pub mod vm;

use std::io::Write;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use std::iter::Iterator;
use std::string::ToString;

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

pub fn dc<I, S, W, E>(mut args: I, stdout: W, stderr: E)
where
    I: Iterator<Item = S>,
    S: AsRef<str> + ToString + PartialEq<str>,
    W: Write,
    E: Write,
{
    let mut program_sources = Vec::new();
    let mut positional_program_sources = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_ref() {
            "-e" | "--expression" => match args.next() {
                Some(text) => program_sources.push(ProgramSource::Text(text.to_string())),
                None => print_help(1),
            },
            "-f" | "--file" => match args.next() {
                Some(file) => program_sources.push(ProgramSource::File(file.to_string())),
                None => print_help(1),
            },
            "-h" | "--help" => print_help(0),
            "-v" | "--version" => print_version(0),
            _ => {
                positional_program_sources.push(ProgramSource::File(arg.to_string()));
            }
        }
    }
    dc_exec_program_sources(program_sources, stdout, stderr);
}

fn dc_exec_program_sources<I, W, E>(program_sources: I, stdout: W, stderr: E)
where
    I: IntoIterator<Item = ProgramSource>,
    W: Write,
    E: Write,
{
    {
        let mut vm = vm::VM::new(stdout, stderr);
        for program_source in program_sources {
            let mut source_code = Vec::new();
            if let Err(error) = program_source.into_bytes(&mut source_code) {
                eprintln!("dc: {}", error);
                continue;
            }
            if let Err(error) = vm.execute(&source_code) {
                eprintln!("dc: {}", error);
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
