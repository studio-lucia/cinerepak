use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::process::exit;

extern crate sega_film;
use sega_film::container::{FILMHeader, Sample};

extern crate clap;
use clap::{Arg, App};

struct AudioData {
    pub buffers : Vec<io::Cursor<Vec<u8>>>,
}

fn copy_sample(start_of_data : usize, sample : &Sample, cpk_data : &[u8], audio : &mut AudioData, output_file : &mut File) -> io::Result<()> {
    let start_offset = sample.offset + start_of_data;

    // Pass through video samples unaltered
    if !sample.is_audio() {
        output_file.write(&cpk_data[start_offset..start_offset + sample.length])?;
        return Ok(());
    }

    let mut left_buf;
    if audio.buffers.len() == 2 {
        left_buf = vec![0; sample.length / 2];
        audio.buffers[0].read(&mut left_buf)?;
        let mut right_buf = vec![0; sample.length / 2];
        audio.buffers[1].read(&mut right_buf)?;
        left_buf.extend(right_buf);
    } else {
        left_buf = vec![0; sample.length];
        audio.buffers[0].read(&mut left_buf)?;
    }
    output_file.write(&left_buf)?;

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

    let mut data;
    let mut input_audio_data = vec![];
    input_audio_file.read_to_end(&mut input_audio_data).unwrap();

    let remux_stereo = header.fdsc.channels == 2 && header.fdsc.audio_codec() == "pcm";
    // Support reformatting stereo audio; this will mangle any other format.
    if remux_stereo {
        // Sega FILM uses a planar audio format, rather than the standard
        // interleaved stereo used by most audio formats.
        // In most audio formats, each pair of left/right audio samples is interleaved.
        // It looks like this: L R L R L R L R
        // In Sega FILM files, each audio chunk instead groups together batches of
        // left/right audio samples. The first half of a chunk contains left samples,
        // and the second half contains right samples. It looks something like this:
        // L L L L R R R R
        // To accommodate that, we need to separate the audio data into left/right
        // segments here so that they can be reformatted into planar chunks as
        // necessary.

        // A pair of 16-bit samples is 4 bytes (2 bytes per sample)
        let chunk_size;
        if header.fdsc.audio_resolution == 16 {
            chunk_size = 4;
        } else {
            chunk_size = 2;
        }

        let left_vec = input_audio_data.chunks(chunk_size)
                                        .flat_map(|bytes| bytes[0..chunk_size / 2].to_vec())
                                        .collect::<Vec<u8>>();
        let left_cursor = io::Cursor::new(left_vec);
        let right_vec = input_audio_data.chunks(chunk_size)
                                        .flat_map(|bytes| bytes[chunk_size / 2..chunk_size].to_vec())
                                        .collect::<Vec<u8>>();
        let right_cursor = io::Cursor::new(right_vec);

        data = AudioData {
            buffers : vec![left_cursor, right_cursor],
        };
    // Pass through audio unaltered.
    } else {
        let cursor = io::Cursor::new(input_audio_data);

        data = AudioData {
            buffers : vec![cursor],
        };
    }

    // OK, first let's copy the header into the output file
    output_file.write(&input_video_buf[0..header.length]).unwrap();
    // Next copy through every sample
    for sample in header.stab.sample_table {
        match copy_sample(header.length, &sample, &input_video_buf, &mut data, &mut output_file) {
            Ok(_) => {},
            Err(e) => {
                println!("Error processing sample at offset {}: {}", sample.offset, e);
                exit(1);
            }
        }
    }
}
