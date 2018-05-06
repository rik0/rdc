extern crate rdc;

use std::io;
use std::process;
use std::ffi::OsStr;
use std::ffi::OsString;

use std::process::Command;

fn run_dc<I, S>(args: I) -> io::Result<process::Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut dc_args = Vec::<OsString>::new();
    for arg in args.into_iter() {
        dc_args.push(OsStr::new("-e").to_os_string());
        dc_args.push(arg.as_ref().to_os_string());
    }

    let mut command = Command::new("dc");
    let c = (&mut command)
        .stdin(process::Stdio::null())
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .args(dc_args.iter());

    let child = c.spawn()?;

    child.wait_with_output()
}

#[test]
fn test() {
    let out = run_dc(vec!["10p"]).expect("process error");
    assert_eq!(
        "10\n",
        String::from_utf8(out.stdout).expect("utf error in output")
    );
}
