use std::path::Path;

use color_eyre::Result;
use image::{GenericImageView, ImageReader};
use image::{ImageBuffer, Luma};
use clap::{Parser};
use std::path::PathBuf;

/// Usage: image_packer <command> [options]
/// Example: image_packer pack input.png output.png
/// Example: image_packer unpack input.png output.png
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub enum Args {
    /// Pack an image to 16-bit ARGB 1555 format.
    Pack {
        /// Input file path
        input: PathBuf,
        /// Output file path
        output: PathBuf,
    },
    /// Unpack a 16-bit ARGB 1555 image to 8-bit RGB.
    Unpack {
        /// Input file path
        input: PathBuf,
        /// Output file path
        output: PathBuf,
    },
}

fn unpack(argb_1555: u16) -> (u8, u8, u8, u8) {
    // Extract individual components from the 16-bit value
    let a = if (argb_1555 >> 15) & 1 == 1 { 255 } else { 0 };
    let r5 = (argb_1555 >> 10) & 0x1F;
    let g5 = (argb_1555 >> 5) & 0x1F;
    let b5 = argb_1555 & 0x1F;

    // Convert 5-bit color values to 8-bit by scaling
    let r = (r5 as u32 * 255 + 15) / 31;
    let g = (g5 as u32 * 255 + 15) / 31;
    let b = (b5 as u32 * 255 + 15) / 31;
    (r as u8, g as u8, b as u8, a)
}

fn pack(r: u8, g: u8, b: u8, a: bool) -> u16 {
    let a_bit = if a { 1 } else { 0 };
    let r5 = (r as u16 * 31 + 127) / 255;
    let g5 = (g as u16 * 31 + 127) / 255;
    let b5 = (b as u16 * 31 + 127) / 255;
    (a_bit << 15) | (r5 << 10) | (g5 << 5) | b5
}

// packs rgb 8-bit image to 16-bit argb 1555
fn pack_image(input_file: &Path, output_file: &Path) -> Result<(), color_eyre::eyre::Error> {
    let img = ImageReader::open(input_file)?.decode()?;
    let (width, height) = img.dimensions();
    let mut argb_1555 = vec![0u16; (width * height) as usize];
    for (x, y, pixel) in img.pixels() {
        let [r, g, b, a] = pixel.0;
        let packed = pack(r, g, b, a > 0);
        argb_1555[(y * width + x) as usize] = packed;
    }
    let out_img = ImageBuffer::<Luma<u16>, Vec<u16>>::from_vec(width, height, argb_1555)
        .expect("Failed to create image buffer");
    out_img.save(output_file)?;
    Ok(())
}

// unpacks 16-bit argb 1555 image to 8-bit rgb
fn unpack_image(input_file: &Path, output_file: &Path) -> Result<(), color_eyre::eyre::Error> {
    let dyn_img = ImageReader::open(input_file)?.decode()?;
    let (width, height) = dyn_img.dimensions();

    let luma_img = match dyn_img {
        image::DynamicImage::ImageLuma16(img) => img,
        _ => return Err(color_eyre::eyre::eyre!("Expected a 16-bit Luma image")),
    };

    let mut rgba_img = image::RgbaImage::new(width, height);
    for (x, y, pixel) in luma_img.enumerate_pixels() {
        let val = pixel.0[0];
        let (r, g, b, a) = unpack(val);
        rgba_img.put_pixel(x, y, image::Rgba([r, g, b, a]));
    }
    rgba_img.save(output_file)?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args {
        Args::Pack { input, output } => pack_image(&input, &output)?,
        Args::Unpack { input, output } => unpack_image(&input, &output)?,
    };

    Ok(())
}
