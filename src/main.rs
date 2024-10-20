use std::{
    fs::{self, File, OpenOptions},
    io::{BufWriter, Seek, SeekFrom, Write},
    path::Path,
};

use anyhow::Context;

struct GdxEntry {
    image_name: String,
    offset: u64,
    size: u64
}

fn write_gdx_entry(gdx_path: &Path, entry: GdxEntry) -> anyhow::Result<()> {
    let mut gdx_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(gdx_path)?;

    writeln!(
        gdx_file,
        "{} {} {}",
        entry.image_name, entry.offset, entry.size
    )?;

    Ok(())
}

fn stream_png_to_gpk(
    png_location: &Path,
    gpk_location: &Path,
    gdx_location: &Path,
    offset: u64,
) -> anyhow::Result<u64> {
    let png_file = File::open(png_location)?;
    let decoder = png::Decoder::new(png_file);

    let mut reader = decoder.read_info()?;
    let info = reader.info().clone();

    let mut buffer = vec![0; reader.output_buffer_size()];

    let mut gpk_file = OpenOptions::new().create(true).append(true).open(gpk_location)?;
    gpk_file.seek(SeekFrom::Start(offset))?;

    let start_pos = gpk_file.stream_position()?;

    let ref mut w = BufWriter::new(gpk_file);
    let encoder = png::Encoder::with_info(&mut *w, info)?;
    let mut writer = encoder.write_header()?;

    let mut counter = 0u8;

    while let Ok(info) = reader.next_frame(&mut buffer) {
        let bytes = &buffer[..info.buffer_size()];
        writer.write_image_data(bytes)?;

        counter += 1;

        println!(" > wrote frame {}", counter);
    }
    
    writer.finish()?;

    let mut gpk_file = w.get_ref();
    let end_pos = gpk_file.stream_position()?;
    let total_bytes_written = end_pos - start_pos;

    let gdx_entry = GdxEntry {
        image_name: png_location
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string(),
        offset,
        size: total_bytes_written,
    };

    write_gdx_entry(gdx_location, gdx_entry)?;

    Ok(total_bytes_written)
}

fn pack_directory(directory: &Path, files_name: &str) -> anyhow::Result<()> {
    let gpk_fname = format!("{}.gpk", files_name);
    let gdx_fname = format!("{}.gdx", files_name);

    let gpk_path = Path::new(&gpk_fname);
    let gdx_path = Path::new(&gdx_fname);

    let mut gpk_file = OpenOptions::new().create(true).append(true).open(gpk_path)?;
    let mut curr_offset = gpk_file.seek(SeekFrom::End(0))?;

    for entry in fs::read_dir(directory).context("failed to read directory")? {
        let entry = entry?;
        let path = entry.path();

        if let Some(ext) = path.extension() {
            if ext == "png" {
                println!("packing png {:?}", path);

                let bytes_written = stream_png_to_gpk(&path, &gpk_path, &gdx_path, curr_offset)?;
                curr_offset += bytes_written;
            }
        }
    }

    Ok(())
}
 
fn main() -> anyhow::Result<()> {
    pack_directory(Path::new("data/"), "datapack")?;
    
    Ok(())
}
