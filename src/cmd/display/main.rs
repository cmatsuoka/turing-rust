use std::error::Error;
use std::process;

use clap::Parser;
use simple_logger::SimpleLogger;

use turing_screen::framebuffer::Framebuffer;
use turing_screen::screen::{Screen, ScreenRevA};

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
    SimpleLogger::new().init()?;

    log::info!("decoding {}", args.image);
    let image = lodepng::decode32_file(args.image)?;
    let mut scr = ScreenRevA::new("AUTO")?;
    let (width, height) = scr.screen_size();

    let mut fb = Framebuffer::new(width, height);

    scr.init()?;
    scr.screen_on()?;
    scr.set_brightness(5)?;

    fb.copy_from(image);
    fb.render_on(&mut scr)?;

    scr.screen_off()?;

    Ok(())
}
