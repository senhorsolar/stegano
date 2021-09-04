use std::env;
use std::fs;
use std::io;
use std::io::{Read, Write};

use image::io::Reader as ImageReader;
use image::{GenericImageView, ImageFormat, save_buffer_with_format};

struct BitReader<'a> {
    bytes: &'a [u8],
    byte_idx: usize,
    bit_idx: usize,
}

impl <'a> BitReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
	BitReader {
	    bytes: bytes,
	    byte_idx: 0,
	    bit_idx: 0,
	}
    }
}

impl Iterator for BitReader<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {

	if self.byte_idx >= self.bytes.len() {
	    return None;
	}

	let cur = (self.bytes[self.byte_idx] >> self.bit_idx) & 1;

	if self.bit_idx == 7 {
	    self.byte_idx += 1;
	}

	self.bit_idx = (self.bit_idx + 1) % 8;

	return Some(cur);
    }
}


fn encode(png_filename: &str, out_filename: &str, data: &[u8]) {

    if data.len() >= 0xffffffff {
	panic!("Encoded data must be less than 2^32-1 (aka 32 bit unsigned int)");
    }
    
    let img = ImageReader::open(png_filename)
	.unwrap()
	.decode()
	.unwrap();

    let (width, height) = img.dimensions();
    let color = img.color();
    let mut pixel_bytes = img.into_bytes();
    
    let bits_to_encode = BitReader::new(data);
    
    if pixel_bytes.len() < (data.len() * 8 + 32)  {
	panic!("PNG file is too small to encode data");
    }
    
    // First encode the size
    for k in 0..32 {
	let bit = ((data.len() >> k) & 1) as u8;
	pixel_bytes[k] |= bit;
    }

    // Now encode data
    for (bit, byte) in bits_to_encode.zip(pixel_bytes[32..].iter_mut()) {
	*byte |= bit;  // set LSB
    }

    save_buffer_with_format(
	out_filename,
	&pixel_bytes,
	width,
	height,
	color,
	ImageFormat::Png
    ).unwrap();    
}

fn decode(png_filename: &str) -> Vec<u8> {
    
    let img = ImageReader::open(png_filename)
	.unwrap()
	.decode()
	.unwrap();
    
    let pixel_bytes = img.into_bytes();
    
    if pixel_bytes.len() < 32 {
	panic!("File too small!");
    }
    
    // Read size
    let mut size: usize = 0;
    for i in 0..32 {
	size += ((pixel_bytes[i] as usize) & 1) << i;
    }

    if pixel_bytes.len() < (32 + size * 8) {
	panic!("File too small!");
    }
    
    let mut decoded_data: Vec<u8> = vec![0; size];

    for i in 0..size {
	for k in 0..8 {
	    decoded_data[i] += (pixel_bytes[i*8 + k + 32] & 1) << k;
	}
    }

    return decoded_data;
}

fn main() {

    let usage_str = "Usage:
                         cargo run -- -e <image.png> <output.png> <file-to-encode>
                         cargo run -- -d <image-to-decode.png> [<output-filename>]";
	
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
	panic!("{}", usage_str);
    }

    let input_png = &args[2];
    
    match args[1].as_str() {
	"-e" => {
	    if args.len() < 4 {
		panic!("{}", usage_str);
	    }

	    let output_png = &args[3];

	    if let Some(data) = match args.len() {
		4 => {  // Check stdin for data to encode
		    let mut buffer = String::new();

		    io::stdin()
			.lock()
			.read_to_string(&mut buffer)
			.expect("failed to read from pipe");
		    
		    Some(buffer.into_bytes())
		}
		5 => {  // Encode file
		    Some(
			fs::read(&args[4]).
			    expect(
				"File containing data to encode doesn't exist"
			    )
		    )
		}
		_ => None
	    } {
		encode(input_png, output_png, &data);
	    }
	    else {
		panic!("{}", usage_str);
	    }   
	}
	"-d" => {

	    let decoded_bytes = decode(input_png);
	    
	    match args.len() {
		3 => {  // Write decoded data to stdout
		    io::stdout()
			.lock()
			.write(&decoded_bytes)
			.expect("Could not write to stdout");
		}
		4 => {  // Write decoded data to file
		    let output_png = &args[3];
		    fs::write(output_png, decoded_bytes)
			.expect("Could not created decoded file");
		}
		_ => panic!("{}", usage_str)
	    };
	}
	_ => {
	    panic!("{}", usage_str)
	}
    };
}
