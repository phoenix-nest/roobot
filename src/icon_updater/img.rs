use std::io::{BufWriter, Cursor};

use base64::prelude::*;
use color_eyre::{eyre::eyre, Result};
use image::{codecs::png::PngEncoder, ImageReader};

pub(crate) fn process_icon(input: Vec<u8>) -> Result<String> {
    let reader = ImageReader::new(Cursor::new(input))
        .with_guessed_format()
        .expect("Cursor IO should never fail");
    if reader.format().is_none() {
        return Err(eyre!("Unsupported image format"));
    }

    let img = reader
        .decode()
        .map_err(|e| eyre!("Could not decode image: {e}"))?;
    let img = img.resize_to_fill(1024, 1024, image::imageops::FilterType::Lanczos3);
    let mut buf = Vec::with_capacity(2_usize.pow(19)); // 512KB
    img.write_with_encoder(PngEncoder::new(BufWriter::new(&mut buf)))
        .map_err(|e| eyre!("Could not encode image to PNG: {e}"))?;
    let payload = format!("data:image/png;base64,{}", BASE64_STANDARD.encode(buf));
    Ok(payload)
}
