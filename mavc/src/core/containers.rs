//! Structural checks for containers whose demuxers can recover past damage.
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

fn read(file: &mut File, bytes: &mut [u8]) -> Result<(), String> {
    file.read_exact(bytes)
        .map_err(|e| format!("truncated container: {e}"))
}

pub(super) fn validate_wav(path: &Path) -> Result<(), String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let len = file.metadata().map_err(|e| e.to_string())?.len();
    let mut header = [0; 12];
    read(&mut file, &mut header)?;
    if &header[..4] != b"RIFF" || &header[8..] != b"WAVE" {
        return Err("expected a RIFF/WAVE file".into());
    }
    let end = 8 + u64::from(u32::from_le_bytes(header[4..8].try_into().unwrap()));
    if end < 12 || end > len {
        return Err("invalid or truncated WAV length".into());
    }
    let mut position = 12;
    let mut alignment = None;
    let mut data_size = None;
    while position < end {
        if end - position < 8 {
            return Err("truncated WAV chunk header".into());
        }
        let mut chunk = [0; 8];
        read(&mut file, &mut chunk)?;
        let size = u64::from(u32::from_le_bytes(chunk[4..].try_into().unwrap()));
        let next = position + 8 + size + size % 2;
        if next > end {
            return Err("truncated WAV chunk payload".into());
        }
        match &chunk[..4] {
            b"fmt " => {
                if alignment.is_some() || size < 16 {
                    return Err("invalid WAV format chunk".into());
                }
                let mut fmt = [0; 16];
                read(&mut file, &mut fmt)?;
                let mut encoding = u16::from_le_bytes(fmt[..2].try_into().unwrap());
                let channels = u16::from_le_bytes(fmt[2..4].try_into().unwrap());
                let rate = u32::from_le_bytes(fmt[4..8].try_into().unwrap());
                let byte_rate = u32::from_le_bytes(fmt[8..12].try_into().unwrap());
                let align = u16::from_le_bytes(fmt[12..14].try_into().unwrap());
                let bits = u16::from_le_bytes(fmt[14..16].try_into().unwrap());
                if encoding == 0xfffe {
                    if size < 40 {
                        return Err("truncated extensible WAV format".into());
                    }
                    let mut ext = [0; 24];
                    read(&mut file, &mut ext)?;
                    if u16::from_le_bytes(ext[..2].try_into().unwrap()) < 22
                        || ext[10..] != [0, 0, 0, 0, 0x10, 0, 0x80, 0, 0, 0xaa, 0, 0x38, 0x9b, 0x71]
                    {
                        return Err("unsupported extensible WAV format".into());
                    }
                    encoding = u16::from_le_bytes(ext[8..10].try_into().unwrap());
                }
                if !matches!(encoding, 1 | 3) {
                    return Err("WAV requires PCM or IEEE float audio".into());
                }
                if channels == 0
                    || rate == 0
                    || bits == 0
                    || bits % 8 != 0
                    || u32::from(align) != u32::from(channels) * u32::from(bits / 8)
                    || u64::from(byte_rate) != u64::from(rate) * u64::from(align)
                {
                    return Err("inconsistent WAV sample format".into());
                }
                alignment = Some(u64::from(align));
            }
            b"data" => {
                if data_size.replace(size).is_some() {
                    return Err("multiple WAV data chunks are unsupported".into());
                }
            }
            _ => {}
        }
        position = next;
        file.seek(SeekFrom::Start(position))
            .map_err(|e| e.to_string())?;
    }
    match (alignment, data_size) {
        (Some(align), Some(size)) if size > 0 && size % align == 0 => Ok(()),
        _ => Err("missing or incomplete WAV samples".into()),
    }
}

pub(super) fn validate_ogg(path: &Path) -> Result<(), String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let len = file.metadata().map_err(|e| e.to_string())?.len();
    let mut position = 0;
    // Serial -> next sequence, unfinished packet, end-of-stream seen.
    let mut streams = HashMap::<u32, (u32, bool, bool)>::new();
    while position < len {
        let mut header = [0; 27];
        read(&mut file, &mut header)?;
        if &header[..4] != b"OggS" || header[4] != 0 || header[5] & !7 != 0 {
            return Err("invalid OGG page header".into());
        }
        let flags = header[5];
        let serial = u32::from_le_bytes(header[14..18].try_into().unwrap());
        let sequence = u32::from_le_bytes(header[18..22].try_into().unwrap());
        let checksum = u32::from_le_bytes(header[22..26].try_into().unwrap());
        let mut segments = vec![0; header[26] as usize];
        read(&mut file, &mut segments)?;
        let size: usize = segments.iter().map(|n| usize::from(*n)).sum();
        let mut payload = vec![0; size];
        read(&mut file, &mut payload)?;
        header[22..26].fill(0);
        let mut crc = 0u32;
        for byte in header.iter().chain(&segments).chain(&payload) {
            crc ^= u32::from(*byte) << 24;
            for _ in 0..8 {
                crc = if crc & 0x80000000 != 0 {
                    (crc << 1) ^ 0x04c11db7
                } else {
                    crc << 1
                };
            }
        }
        if crc != checksum {
            return Err("OGG page checksum verification failed".into());
        }
        if let Some((next, continued, ended)) = streams.get_mut(&serial) {
            if *ended || flags & 2 != 0 || sequence != *next || (flags & 1 != 0) != *continued {
                return Err("invalid OGG page sequence or packet continuation".into());
            }
            *next = sequence.wrapping_add(1);
            if let Some(last) = segments.last() {
                *continued = *last == 255;
            }
            *ended = flags & 4 != 0;
            if *ended && *continued {
                return Err("incomplete final OGG packet".into());
            }
        } else {
            if flags & 2 == 0 || flags & 1 != 0 || sequence != 0 {
                return Err("missing OGG beginning-of-stream page".into());
            }
            if !streams.is_empty() {
                return Err("multiple or chained OGG streams are unsupported".into());
            }
            streams.insert(serial, (1, segments.last() == Some(&255), flags & 4 != 0));
        }
        position += 27 + segments.len() as u64 + size as u64;
    }
    if streams.is_empty()
        || streams
            .values()
            .any(|(_, continued, ended)| *continued || !ended)
    {
        return Err("missing OGG end-of-stream page".into());
    }
    Ok(())
}
