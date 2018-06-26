extern crate rdc;


use std::error::Error;

use rdc::dcnumber::traits::FromBytes;

fn main() -> Result<(), Box<Error>> {
    // let us implement the real app to understand approaches to ownership

    let mut args = std::env::args().skip(1);

    let iterations: u32 = args.next()
        .expect("no args")
        .parse()?;

    args.for_each(|n| {
        for _ in 0..iterations {
            let _ = rdc::dcnumber::unsigned::UnsignedDCNumber::from_bytes(n.as_ref() );
        }
    });

    Ok(())
}