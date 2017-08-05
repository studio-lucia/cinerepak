fn uint32_from_bytes(bytes : [u8; 4]) -> u32 {
    return ((bytes[0] as u32) << 24) +
        ((bytes[1] as u32) << 16) +
        ((bytes[2] as u32) << 8) +
        bytes[3] as u32;
}

fn uint16_from_bytes(bytes : [u8; 2]) -> u16 {
    return ((bytes[0] as u16) << 8) + bytes[1] as u16;
}

struct FILMHeader {
    // Always 'FILM'
    signature: String,
    length: usize,
    version: String,
    unknown: Vec<u8>,
    fdsc: FDSC,
    stab: STAB,
}

impl FILMHeader {
    fn parse(data : &[u8]) -> FILMHeader {
        let length = uint32_from_bytes([data[4], data[5], data[6], data[7]]) as usize;
        return FILMHeader {
            signature: String::from_utf8(data[0..3].to_vec()).unwrap(),
            length: length,
            version: String::from_utf8(data[8..11].to_vec()).unwrap(),
            unknown: data[12..15].to_vec(),
            fdsc: FDSC::parse(&data[16..47]),
            stab: STAB::parse(&data[48..length]),
        }
    }
}

struct FDSC {
    // Always 'FDSC'
    signature: String,
    length: u32,
    fourcc: String,
    height: u32,
    width: u32,
    // In practice always 24
    bpp: u8,
    channels: u8,
    // Always 8 or 32
    audio_resolution: u8,
    audio_compression: u8,
    audio_sampling_rate: u16,
}

impl FDSC {
    pub fn parse(data : &[u8]) -> FDSC {
        let signature_bytes = vec![
            data[0], data[1], data[2], data[3],
        ];
        let fourcc_bytes = vec![
            data[8], data[9], data[10], data[11],
        ];

        return FDSC {
            signature: String::from_utf8(signature_bytes).unwrap(),
            length: uint32_from_bytes([data[4], data[5], data[6], data[7]]),
            fourcc: String::from_utf8(fourcc_bytes).unwrap(),
            height: uint32_from_bytes([data[12], data[13], data[14], data[15]]),
            width: uint32_from_bytes([data[16], data[17], data[18], data[19]]),
            bpp: data[20],
            channels: data[21],
            audio_resolution: data[22],
            audio_compression: data[23],
            audio_sampling_rate: uint16_from_bytes([data[24], data[25]]),
        };
    }
}

struct STAB {
    // Always 'STAB'
    signature: String,
    length: u32,
    // in Hz
    framerate: u32,
    // Number of entries in the sample table
    entries: u32,
    sample_table: Vec<Sample>,
}

impl STAB {
    pub fn parse(data : &[u8]) -> STAB {
        let signature_bytes = vec![
            data[0], data[1], data[2], data[3],
        ];
        let entries = uint32_from_bytes([data[12], data[13], data[14], data[15]]);
        let mut samples = vec![];
        for i in 1..entries {
            let index = i as usize * 16;
            let sample = Sample::parse(&data[index..index + 16]);
            samples.push(sample);
        }

        return STAB {
            signature: String::from_utf8(signature_bytes).unwrap(),
            length: uint32_from_bytes([data[4], data[5], data[6], data[7]]),
            framerate: uint32_from_bytes([data[8], data[9], data[10], data[11]]),
            entries: entries,
            sample_table: samples,
        };
    }
}

struct Sample {
    offset: u32,
    length: u32,
    info1: [u8; 4],
    info2: [u8; 4],
}

impl Sample {
    pub fn parse(data : &[u8]) -> Sample {
        return Sample {
            offset: uint32_from_bytes([data[0], data[1], data[2], data[3]]),
            length: uint32_from_bytes([data[4], data[5], data[6], data[7]]),
            info1: [data[8], data[9], data[10], data[11]],
            info2: [data[12], data[13], data[14], data[15]],
        }
    }

    // For the purpose of this program, we don't care about video data at all;
    // we just want to be able to identify which samples are audio.
    pub fn is_audio(&self) -> bool {
        return self.info1 == [1, 1, 1, 1];
    }
}
