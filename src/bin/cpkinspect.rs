use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::exit;

extern crate cinerepak;
use cinerepak::FILMHeader;

extern crate clap;
use clap::{Arg, App};

fn print_header_info(filename : &str, header : &FILMHeader) {
    println!("File: {}", filename);
    println!("");

    println!("Container:");
    println!("Version: {}", header.version);
    println!("");

    println!("Video:");
    println!("Format: {}", header.fdsc.human_readable_fourcc());
    println!("Resolution: {}x{}", header.fdsc.width, header.fdsc.height);
    println!("Bits per pixel: {}", header.fdsc.bpp);
    println!("Ticks per second: {}", header.stab.framerate);
    println!("");

    println!("Audio:");
    println!("Format: {}", header.fdsc.audio_codec().to_uppercase());
    println!("Bit rate: {}", header.fdsc.audio_resolution);
    println!("Sampling rate: {} Hz", header.fdsc.audio_sampling_rate);
    println!("");
}

fn main() {
    let matches = App::new("cpkinspect")
                          .version("0.1.0")
                          .author("Misty De Meo")
                          .about("Display Sega FILM metadata")
                          .arg(Arg::with_name("input")
                              .help("Script files to process")
                              .required(true))
                          .get_matches();
    let input = matches.value_of("input").unwrap();
    let input_path = Path::new(input);
    if !input_path.exists() {
        println!("Input file {} does not exist!", input);
        exit(1);
    }

    let mut input_file;
    match File::open(input_path) {
        Ok(f) => input_file = f,
        Err(e) => {
            println!("Error reading input file {}: {}", input, e);
            exit(1);
        }
    }
    // This is obviously too big to read at once,
    // but the header parser assumes you have all the data up front,
    // and the sample table is legitimately quite big.
    // Will fix this later, maybe, probably.
    let mut buffer = vec![0; 128_000];
    input_file.read(&mut buffer).unwrap();

    let header = FILMHeader::parse(&buffer);
    print_header_info(&input, &header);
}
