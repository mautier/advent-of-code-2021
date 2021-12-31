//! Helpers for reading/writing Netpbm-format images.
//! https://en.wikipedia.org/wiki/Netpbm

use crate::colormap::Rgb;
use crate::image::Image;

/// Saves an RGB image as a Portable PixMap (PPM), in binary format (format P6).
pub fn save_image_as_ppm(img: &Image<Rgb>, path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Write;
    let mut writer = std::io::BufWriter::new(std::fs::File::create(path)?);

    // Magic number for a binary PPM.
    writer.write_all(b"P6\n")?;
    // Width, then height, as ASCII.
    writeln!(&mut writer, "{} {}", img.width, img.height)?;
    // The maximum value for each color.
    writeln!(&mut writer, "255")?;
    // Now the binary pixel data.
    for px in &img.data {
        writer.write_all(&[px.r(), px.g(), px.b()])?;
    }

    writer.flush()
}

/// Reads a grayscale ([0, 255]) image in Portable GrayMap ASCII format (PGM P2).
pub fn read_pgm_image(path: &std::path::Path) -> std::io::Result<Image<u8>> {
    let contents = std::fs::read_to_string(path)?;
    let mut contents = contents.as_str();

    contents = contents
        .strip_prefix("P2")
        .expect("Missing P2 magic header prefix.");

    let mut height = None;
    let mut width = None;
    let mut data = Vec::new();

    #[derive(Eq, PartialEq)]
    enum Mode {
        Width,
        Height,
        MaxPixelValue,
        Pixels,
    }
    let mut mode = Mode::Width;

    loop {
        // Remove whitespace.
        contents = contents.trim_start();

        if contents.is_empty() {
            break;
        }

        if contents.starts_with('#') {
            // Found a comment line, such as the one Gimp inserts. Read till end of line and
            // discard.
            let next_linefeed = contents.find('\n').unwrap_or(contents.len());
            contents = &contents[next_linefeed..];
            continue;
        }
        match mode {
            Mode::Width => {
                let next_ws = contents.find(char::is_whitespace).unwrap();
                width = Some(contents[..next_ws].parse::<u16>().unwrap());
                contents = &contents[next_ws..];
                mode = Mode::Height;
            }
            Mode::Height => {
                let next_ws = contents.find(char::is_whitespace).unwrap();
                height = Some(contents[..next_ws].parse::<u16>().unwrap());
                contents = &contents[next_ws..];
                mode = Mode::MaxPixelValue;
            }
            Mode::MaxPixelValue => {
                let next_ws = contents.find(char::is_whitespace).unwrap();
                let max_val = contents[..next_ws].parse::<u16>().unwrap();
                assert!(max_val <= 255);
                contents = &contents[next_ws..];
                mode = Mode::Pixels;
            }
            Mode::Pixels => {
                let next_ws = contents.find(char::is_whitespace).unwrap_or(contents.len());
                let px = contents[..next_ws].parse::<u8>().unwrap();
                data.push(px);
                contents = &contents[next_ws..];
            }
        }
    }

    if mode != Mode::Pixels {
        panic!("File ended before pixel data.");
    }

    let height = height.unwrap();
    let width = width.unwrap();
    if data.len() != height as usize * width as usize {
        panic!(
            "Parsed an inconsistent number of pixels. Expected {}, parsed {}",
            height as usize * width as usize,
            data.len()
        );
    }

    Ok(Image {
        height,
        width,
        data,
    })
}
