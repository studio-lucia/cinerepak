use std::fs::File;
use std::io::Read;
use std::io::{Seek, SeekFrom};
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
                              .help("CPK file to inspect")
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

    // First, we read the first 8 bytes to determine
    // a) is this a Sega FILM file?, and
    // b) how long is the header?
    // The latter is variable-length, so this saves us from
    // naively reading way too many bytes off the top.
    let mut header_buffer = vec![0; 8];
    input_file.read(&mut header_buffer).unwrap();
    if !FILMHeader::is_film_file(&header_buffer) {
        println!("Input file {} is not a valid Sega FILM file!", input);
        exit(1);
    }
    let header_length = FILMHeader::guess_length(&header_buffer);

    let mut buffer = vec![0; header_length];
    // Since we previously read 8 bytes off the top
    input_file.seek(SeekFrom::Start(0)).unwrap();
    input_file.read(&mut buffer).unwrap();

    let header;
    match FILMHeader::parse(&buffer) {
        Ok(h) => header = h,
        Err(e) => {
            println!("Encountered an error processing file {}:", input);
            println!("{}", e);
            exit(1);
        }
    }
    print_header_info(&input, &header);
}
