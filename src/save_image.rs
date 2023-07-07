use crate::sphere::Color;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn save_image(
    image: &Vec<Vec<Color>>,
    width: u32,
    height: u32,
    filename: &str,
) -> std::io::Result<()> {
    let path: &Path = Path::new(filename);
    let mut file: File = File::create(&path)?;

    write!(file, "P3\n{} {}\n255\n", width, height)?;

    for row in image {
        for color in row {
            let r: u8 = (color.r.min(1.0).max(0.0) * 255.0) as u8;
            let g: u8 = (color.g.min(1.0).max(0.0) * 255.0) as u8;
            let b: u8 = (color.b.min(1.0).max(0.0) * 255.0) as u8;
            write!(file, "{} {} {}\n", r, g, b)?;
        }
    }
    Ok(())
}
