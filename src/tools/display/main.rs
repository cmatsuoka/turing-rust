use std::error::Error;
use std::process;

use clap::Parser;

#[derive(Parser)]
#[command(name = "turing-screen")]
#[command(about = "A lightweight turing smart screen updater")]
struct Args {
    /// Serial device to use
    #[arg(short, long, value_name = "device", default_value_t = String::from("AUTO"))]
    port: String,

    #[arg(value_name = "filename")]
    image: String,
}

fn main() {
    let args = Args::parse();

    match run(args) {
        Ok(_) => (),
        Err(err) => {
            eprintln!("error: {err}");
            process::exit(1);
        }
    }
}

fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let _image = lodepng::decode32_file(args.image)?;

    Ok(())
}
