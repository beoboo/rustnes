//! Minimal WAV writer.
//!
//! Hand-rolled rather than pulling in a crate: a 16-bit PCM header is 44 bytes and this way the
//! file format is not a black box when a capture looks wrong.

use std::{fs::File, io::Write, path::Path};

use anyhow::Result;

pub fn write_mono_16bit(path: &Path, samples: &[f32], sample_rate: u32) -> Result<()> {
    let mut file = File::create(path)?;

    let data_len = samples.len() as u32 * 2; // 16 bits per sample
    let byte_rate = sample_rate * 2;

    // RIFF header
    file.write_all(b"RIFF")?;
    file.write_all(&(36 + data_len).to_le_bytes())?;
    file.write_all(b"WAVE")?;

    // fmt chunk
    file.write_all(b"fmt ")?;
    file.write_all(&16u32.to_le_bytes())?; // chunk size
    file.write_all(&1u16.to_le_bytes())?; // PCM
    file.write_all(&1u16.to_le_bytes())?; // mono
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&2u16.to_le_bytes())?; // block align
    file.write_all(&16u16.to_le_bytes())?; // bits per sample

    // data chunk
    file.write_all(b"data")?;
    file.write_all(&data_len.to_le_bytes())?;
    for &sample in samples {
        // Clamp before converting: the APU should never exceed full scale, but a bug that made it
        // do so must show up as visible clipping rather than as wraparound noise.
        let value = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        file.write_all(&value.to_le_bytes())?;
    }

    Ok(())
}
