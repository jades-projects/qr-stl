use std::fs;
use std::io;
use std::io::Read;
use std::path::PathBuf;

use structopt::StructOpt;

use qr_stl::Result;
use qr_stl::{qr_to_triangles, save_stl, MeshOptions};

#[derive(StructOpt, Debug)]
#[structopt(name = "qr-stl")]
struct Opt {
    /// size in units of pixels in the generated qr code
    #[structopt(long, default_value = "2.5")]
    pixel_size: f32,

    /// width of the base to put on the qr code
    #[structopt(long, default_value = "5.0")]
    base_size: f32,

    /// height of the base to put on the qr code
    #[structopt(long, default_value = "3.0")]
    base_height: f32,

    /// input text file for the qr content
    #[structopt(short = "i", long, parse(from_os_str))]
    input: Option<PathBuf>,

    /// output file path
    #[structopt(short = "o", long, parse(from_os_str))]
    output: PathBuf,
}

fn main() -> Result<()> {
    let opts = Opt::from_args();

    let mut in_file: Box<dyn Read> = match opts.input {
        Some(f) => Box::new(fs::OpenOptions::new().read(true).open(f)?),
        None => Box::new(io::stdin()),
    };

    let mut input = Vec::new();
    in_file.read_to_end(&mut input)?;
    println!("Generating triangles...");

    let tris = qr_to_triangles(
        &input,
        &MeshOptions {
            base_height: opts.base_height,
            base_size: opts.base_size,
            pixel_size: opts.pixel_size,
        },
    )?;

    let mut out_file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(opts.output)?;
    println!("Writing STL...");
    save_stl(&tris, &mut out_file)?;

    Ok(())
}
