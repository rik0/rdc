extern crate rdc;

fn main() {
    // let us implement the real app to understand approaches to ownership

    let args = std::env::args().skip(1);

    rdc::dc(args, std::io::stdout(), std::io::stderr());
}
