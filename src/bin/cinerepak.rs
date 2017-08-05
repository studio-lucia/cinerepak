use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::process::exit;

extern crate cinerepak;
use cinerepak::{FILMHeader, Sample};

extern crate clap;
use clap::{Arg, App};

fn copy_sample(start_of_data : usize, sample : &Sample, cpk_data : &[u8], audio_file : &mut File, output_file : &mut File) -> io::Result<()> {
    let start_offset = sample.offset + start_of_data;

    // Pass through video samples unaltered
    if !sample.is_audio() {
        output_file.write(&cpk_data[start_offset..start_offset + sample.length])?;
        return Ok(());
    }

    let mut buf = vec![0; sample.length];
    audio_file.read_to_end(&mut buf)?;
    output_file.write(&buf)?;

    return Ok(());
}

fn main() {
    let matches = App::new("cpkinspect")
                          .version("0.1.0")
                          .author("Misty De Meo")
                          .about("Display Sega FILM metadata")
                          .arg(Arg::with_name("input")
                              .help("CPK file to process")
                              .required(true))
                          .arg(Arg::with_name("input_audio")
                              .help("New audio track")
                              .required(true))
                          .arg(Arg::with_name("output")
                              .help("Output file name")
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

    let input_audio = matches.value_of("input_audio").unwrap();
    let input_audio_path = Path::new(input_audio);
    if !input_audio_path.exists() {
        println!("Input file {} does not exist!", input);
        exit(1);
    }
    let mut input_audio_file;
    match File::open(input_audio_path) {
        Ok(f) => input_audio_file = f,
        Err(e) => {
            println!("Error reading input audio file {}: {}", input, e);
            exit(1);
        }
    }

    let output = matches.value_of("output").unwrap();
    let mut output_file;
    match File::create(output) {
        Ok(f) => output_file = f,
        Err(e) => {
            println!("Error creating output file {}: {}", output, e);
            exit(1);
        }
    }

    // Obviously we're not going to keep the whole video in RAM going forward
    let mut input_video_buf = vec![];
    input_file.read_to_end(&mut input_video_buf).unwrap();
    let header;
    match FILMHeader::parse(&input_video_buf) {
        Ok(h) => header = h,
        Err(e) => {
            println!("Encountered an error processing file {}:", input);
            println!("{}", e);
            exit(1);
        }
    }

    // OK, first let's copy the header into the output file
    output_file.write(&input_video_buf[0..header.length]).unwrap();
    // Next copy through every sample
    for sample in header.stab.sample_table {
        match copy_sample(header.length, &sample, &input_video_buf, &mut input_audio_file, &mut output_file) {
            Ok(_) => {},
            Err(e) => {
                println!("Error processing sample at offset {}: {}", sample.offset, e);
                exit(1);
            }
        }
    }
}
