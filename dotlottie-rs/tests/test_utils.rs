pub const WIDTH: u32 = 512;
pub const HEIGHT: u32 = 512;

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

/// Writes the raw RGBA buffer to a file for snapshot testing
///
/// # Arguments
/// * `buffer` - The buffer pointer (u32 per pixel, RGBA format)
/// * `width` - Width of the image
/// * `height` - Height of the image
/// * `path` - Path where the snapshot file should be saved
pub fn write_buffer_snapshot<P: AsRef<Path>>(
    buffer: *const u32,
    width: u32,
    height: u32,
    path: P,
) -> io::Result<()> {
    let pixel_count = (width * height) as usize;

    // Safety: Assuming the buffer pointer is valid and contains pixel_count u32 values
    let buffer_slice = unsafe { std::slice::from_raw_parts(buffer, pixel_count) };

    let mut file = File::create(path)?;

    // Write dimensions first (as metadata)
    file.write_all(&width.to_le_bytes())?;
    file.write_all(&height.to_le_bytes())?;

    // Convert u32 pixels to bytes and write
    let byte_buffer: Vec<u8> = buffer_slice
        .iter()
        .flat_map(|&pixel| pixel.to_le_bytes())
        .collect();

    file.write_all(&byte_buffer)?;

    Ok(())
}

/// Reads a buffer snapshot from a file
///
/// # Arguments
/// * `path` - Path to the snapshot file
///
/// # Returns
/// A tuple of (buffer_data as u32 pixels, width, height)
pub fn read_buffer_snapshot<P: AsRef<Path>>(path: P) -> io::Result<(Vec<u32>, u32, u32)> {
    let mut file = File::open(path)?;

    // Read dimensions
    let mut width_bytes = [0u8; 4];
    let mut height_bytes = [0u8; 4];

    file.read_exact(&mut width_bytes)?;
    file.read_exact(&mut height_bytes)?;

    let width = u32::from_le_bytes(width_bytes);
    let height = u32::from_le_bytes(height_bytes);

    // Read buffer data as bytes
    let pixel_count = (width * height) as usize;
    let byte_count = pixel_count * 4;
    let mut byte_buffer = vec![0u8; byte_count];
    file.read_exact(&mut byte_buffer)?;

    // Convert bytes back to u32 pixels
    let buffer: Vec<u32> = byte_buffer
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    Ok((buffer, width, height))
}

/// Compares two buffers for equality
///
/// # Arguments
/// * `buffer1` - First buffer pointer (u32 pixels)
/// * `buffer2` - Second buffer pointer (u32 pixels)
/// * `width` - Width of the images
/// * `height` - Height of the images
///
/// # Returns
/// true if buffers are identical, false otherwise
pub fn compare_buffers(buffer1: *const u32, buffer2: *const u32, width: u32, height: u32) -> bool {
    let pixel_count = (width * height) as usize;

    let slice1 = unsafe { std::slice::from_raw_parts(buffer1, pixel_count) };
    let slice2 = unsafe { std::slice::from_raw_parts(buffer2, pixel_count) };

    slice1 == slice2
}

/// Compares a buffer with a snapshot file
///
/// # Arguments
/// * `buffer` - Current buffer pointer (u32 pixels)
/// * `width` - Width of the current buffer
/// * `height` - Height of the current buffer
/// * `snapshot_path` - Path to the snapshot file to compare against
///
/// # Returns
/// Result indicating if buffers match, or an error if reading fails
pub fn compare_with_snapshot<P: AsRef<Path>>(
    buffer: *const u32,
    width: u32,
    height: u32,
    snapshot_path: P,
) -> io::Result<bool> {
    let (snapshot_buffer, snapshot_width, snapshot_height) = read_buffer_snapshot(snapshot_path)?;

    // Check dimensions match
    if width != snapshot_width || height != snapshot_height {
        return Ok(false);
    }

    Ok(compare_buffers(
        buffer,
        snapshot_buffer.as_ptr(),
        width,
        height,
    ))
}

/// Helper function to get detailed pixel differences (useful for debugging)
///
/// # Returns
/// Vector of (pixel_index, expected_value, actual_value) for differing pixels
pub fn get_buffer_diff(
    buffer: *const u32,
    width: u32,
    height: u32,
    snapshot_path: impl AsRef<Path>,
) -> io::Result<Vec<(usize, u32, u32)>> {
    let (snapshot_buffer, snapshot_width, snapshot_height) = read_buffer_snapshot(snapshot_path)?;

    if width != snapshot_width || height != snapshot_height {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Dimension mismatch: {}x{} vs {}x{}",
                width, height, snapshot_width, snapshot_height
            ),
        ));
    }

    let pixel_count = (width * height) as usize;
    let slice1 = unsafe { std::slice::from_raw_parts(buffer, pixel_count) };

    let differences: Vec<(usize, u32, u32)> = slice1
        .iter()
        .zip(snapshot_buffer.iter())
        .enumerate()
        .filter_map(|(idx, (&actual, &expected))| {
            if actual != expected {
                Some((idx, expected, actual))
            } else {
                None
            }
        })
        .collect();

    Ok(differences)
}

/// Converts a .bin snapshot file to a PNG image
///
/// # Arguments
/// * `snapshot_path` - Path to the .bin snapshot file
/// * `output_path` - Path where the PNG image should be saved
pub fn snapshot_to_png<P: AsRef<Path>, Q: AsRef<Path>>(
    snapshot_path: P,
    output_path: Q,
) -> io::Result<()> {
    let (buffer, width, height) = read_buffer_snapshot(snapshot_path)?;

    // Convert u32 ABGR8888 to u8 RGBA bytes for PNG
    let rgba_bytes: Vec<u8> = buffer
        .iter()
        .flat_map(|&pixel| {
            // ABGR8888 format: A at position 24, B at 16, G at 8, R at 0
            let r = (pixel & 0xFF) as u8;
            let g = ((pixel >> 8) & 0xFF) as u8;
            let b = ((pixel >> 16) & 0xFF) as u8;
            let a = ((pixel >> 24) & 0xFF) as u8;

            [r, g, b, a]
        })
        .collect();

    // Create PNG encoder
    let file = File::create(output_path)?;
    let w = &mut std::io::BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    // Explicitly set compression
    encoder.set_compression(png::Compression::Default);

    let mut writer = encoder
        .write_header()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    writer
        .write_image_data(&rgba_bytes)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(())
}

/// Diagnostic function to analyze the alpha channel in a snapshot
pub fn analyze_snapshot_alpha<P: AsRef<Path>>(snapshot_path: P) -> io::Result<()> {
    let (buffer, width, height) = read_buffer_snapshot(snapshot_path)?;

    let mut fully_transparent = 0;
    let mut fully_opaque = 0;
    let mut semi_transparent = 0;
    let mut alpha_values: std::collections::HashMap<u8, usize> = std::collections::HashMap::new();

    for pixel in buffer.iter() {
        let alpha = ((pixel >> 24) & 0xFF) as u8;

        *alpha_values.entry(alpha).or_insert(0) += 1;

        match alpha {
            0 => fully_transparent += 1,
            255 => fully_opaque += 1,
            _ => semi_transparent += 1,
        }
    }

    println!("=== Alpha Channel Analysis ===");
    println!("Dimensions: {}x{}", width, height);
    println!("Total pixels: {}", width * height);
    println!("Fully transparent (alpha=0): {}", fully_transparent);
    println!("Semi-transparent (0<alpha<255): {}", semi_transparent);
    println!("Fully opaque (alpha=255): {}", fully_opaque);
    println!("\nAlpha value distribution:");

    let mut alpha_sorted: Vec<_> = alpha_values.iter().collect();
    alpha_sorted.sort_by_key(|(alpha, _)| *alpha);

    for (alpha, count) in alpha_sorted.iter().take(10) {
        println!("  Alpha {}: {} pixels", alpha, count);
    }

    Ok(())
}

/// Compares with snapshot, creating it if it doesn't exist
pub fn compare_or_create_snapshot<P: AsRef<Path>>(
    buffer: *const u32,
    width: u32,
    height: u32,
    snapshot_path: P,
) -> io::Result<bool> {
    let path = snapshot_path.as_ref();

    if !path.exists() {
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Create the snapshot
        write_buffer_snapshot(buffer, width, height, path)?;
        println!("Created new snapshot: {}", path.display());
        return Ok(true);
    }

    // Compare with existing snapshot
    compare_with_snapshot(buffer, width, height, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_roundtrip() {
        let width = 100u32;
        let height = 100u32;
        let pixel_count = (width * height) as usize;
        let test_buffer: Vec<u32> = (0..pixel_count).map(|i| i as u32).collect();

        let temp_path = "test_snapshot.bin";

        // Write snapshot
        write_buffer_snapshot(test_buffer.as_ptr(), width, height, temp_path).unwrap();

        // Read snapshot
        let (read_buffer, read_width, read_height) = read_buffer_snapshot(temp_path).unwrap();

        // Verify
        assert_eq!(width, read_width);
        assert_eq!(height, read_height);
        assert_eq!(test_buffer, read_buffer);

        // Clean up
        std::fs::remove_file(temp_path).unwrap();
    }
}
