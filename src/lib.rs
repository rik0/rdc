extern crate bigdecimal;
extern crate num;
extern crate num_bigint;

mod instructions;
mod parse;
#[macro_use]
mod dcstack;
pub mod vm;

use std::io::Write;
use std::io;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use std::iter::Iterator;
use std::ffi::OsStr;
use std::ops::Deref;

enum ProgramSource<ProgramText, ProgramPath> {
    Text(ProgramText),
    File(ProgramPath),
}

impl<ProgramText, ProgramPath> ProgramSource<ProgramText, ProgramPath>
where
    ProgramText: Deref<Target = str>,
    ProgramPath: AsRef<OsStr>,
{
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

fn parse_args<ProgramText, ProgramPath, I, S>(
    mut args: I,
) -> Vec<ProgramSource<ProgramText, ProgramPath>>
where
    I: Iterator<Item = S>,
    S: AsRef<str> + Into<ProgramText> + Into<ProgramPath>,
{
    let mut program_sources: Vec<ProgramSource<ProgramText, ProgramPath>> = Vec::new();
    let mut positional_program_sources: Vec<ProgramSource<ProgramText, ProgramPath>> = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_ref() {
            "-e" | "--expression" => match args.next() {
                Some(text) => program_sources.push(ProgramSource::Text(text.into())),
                None => print_help(1),
            },
            "-f" | "--file" => match args.next() {
                Some(file) => program_sources.push(ProgramSource::File(file.into())),
                None => print_help(1),
            },
            "-h" | "--help" => print_help(0),
            "-v" | "--version" => print_version(0),
            _ => {
                positional_program_sources.push(ProgramSource::File(arg.into()));
            }
        }
    }
    program_sources.extend(positional_program_sources);
    program_sources
}

pub fn dc<'a, I, S, W, E>(args: I, stdout: W, stderr: E)
where
    I: Iterator<Item = S>,
    S: AsRef<str> + Into<String> + PartialEq<&'a str>,
    W: Write,
    E: Write,
{
    let program_sources = parse_args(args);
    dc_exec_program_sources(program_sources, stdout, stderr);
}

fn dc_exec_program_sources<ProgramText, ProgramPath, I, W, E>(
    program_sources: I,
    stdout: W,
    stderr: E,
) where
    I: IntoIterator<Item = ProgramSource<ProgramText, ProgramPath>>,
    ProgramPath: AsRef<OsStr>,
    ProgramText: Deref<Target = str>,
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
