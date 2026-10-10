//! Validate imported audio before an archive is created or rewritten.
use super::{error::Error, metadata::codec_for};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};
use symphonia::core::{
    codecs::{CODEC_TYPE_AAC, CODEC_TYPE_FLAC, CODEC_TYPE_MP3, CODEC_TYPE_VORBIS, DecoderOptions},
    errors::Error as DecodeError,
    formats::FormatOptions,
    io::{MediaSource, MediaSourceStream},
    meta::MetadataOptions,
    probe::Hint,
};

pub(super) fn validate_audio(path: &Path) -> Result<(), Error> {
    let extension = codec_for(path)?;
    let result = validate_audio_stream(path, extension, None);
    if result.is_err() && extension == "flac" {
        // Some encoders append non-audio bytes. Symphonia includes those bytes
        // in the last packet, fails its CRC and silently drops the whole frame.
        // Retry at a CRC-valid boundary; the complete sample count and PCM MD5
        // still have to pass, so this cannot forgive missing audio samples.
        if let Some(end) = flac_audio_end(path)? {
            if validate_audio_stream(path, extension, Some(end)).is_ok() {
                return Ok(());
            }
        }
    }
    result
}

fn validate_audio_stream(path: &Path, extension: &str, end: Option<u64>) -> Result<(), Error> {
    let invalid = |reason: &str| {
        Error::Invalid(format!(
            "cannot import '{}': damaged or invalid audio ({reason})",
            path.display()
        ))
    };
    let mut file = File::open(path).map_err(|error| invalid(&error.to_string()))?;
    if file.metadata()?.len() == 0 {
        return Err(invalid("empty file"));
    }
    // Demuxers may silently resynchronize or treat an incomplete last packet as
    // EOF. Check raw MP3/ADTS boundaries separately so those errors are rejected.
    let (start, audio_end) = if matches!(extension, "mp3" | "aac") {
        validate_frames(path, extension).map_err(|reason| invalid(&reason))?
    } else {
        (0, file.metadata()?.len())
    };
    match extension {
        "wav" => super::containers::validate_wav(path).map_err(|reason| invalid(&reason))?,
        "ogg" => super::containers::validate_ogg(path).map_err(|reason| invalid(&reason))?,
        _ => {}
    }
    let len = end.unwrap_or(audio_end) - start;
    file.seek(SeekFrom::Start(start))?;
    let stream =
        MediaSourceStream::new(Box::new(AudioFile { file, start, len }), Default::default());
    let mut hint = Hint::new();
    hint.with_extension(extension);
    let mut format = symphonia::default::get_probe()
        .format(
            &hint,
            stream,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|error| invalid(&error.to_string()))?
        .format;
    let track = format
        .default_track()
        .ok_or_else(|| invalid("no audio track"))?;
    let expected_codec = match extension {
        "mp3" => CODEC_TYPE_MP3,
        "flac" => CODEC_TYPE_FLAC,
        "aac" => CODEC_TYPE_AAC,
        "ogg" => CODEC_TYPE_VORBIS,
        "wav" => track.codec_params.codec,
        _ => unreachable!(),
    };
    if track.codec_params.codec != expected_codec {
        return Err(invalid("audio content does not match its file extension"));
    }
    let id = track.id;
    let expected_frames = if matches!(extension, "flac" | "wav") {
        track.codec_params.n_frames
    } else {
        None
    };
    if end.is_some() && !expected_frames.is_some_and(|count| count > 0) {
        return Err(invalid("cannot verify audio length before trailing data"));
    }
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions { verify: true })
        .map_err(|error| invalid(&error.to_string()))?;
    let mut frames = 0u64;
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(DecodeError::IoError(error))
                if error.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(error) => return Err(invalid(&error.to_string())),
        };
        if packet.track_id() != id {
            continue;
        }
        let audio = decoder
            .decode(&packet)
            .map_err(|error| invalid(&error.to_string()))?;
        frames = frames
            .checked_add(audio.frames() as u64)
            .ok_or_else(|| invalid("sample count overflow"))?;
    }
    if frames == 0 {
        return Err(invalid("no decodable audio samples"));
    }
    if expected_frames.is_some_and(|expected| expected != frames) {
        return Err(invalid(&format!(
            "decoded sample count ({frames}) does not match the declared length ({expected_frames:?}); audio may be truncated"
        )));
    }
    if decoder.finalize().verify_ok == Some(false) {
        return Err(invalid("audio checksum verification failed"));
    }
    Ok(())
}

fn read_at(file: &mut File, position: u64, bytes: &mut [u8]) -> Result<(), String> {
    file.seek(SeekFrom::Start(position))
        .map_err(|e| e.to_string())?;
    file.read_exact(bytes)
        .map_err(|e| format!("incomplete audio frame: {e}"))
}

fn validate_frames(path: &Path, codec: &str) -> Result<(u64, u64), String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let len = file.metadata().map_err(|e| e.to_string())?.len();
    let mut end = len;
    // ID3v1 and APEv2 tags are metadata, not audio frames.
    if end >= 128 {
        let mut marker = [0; 3];
        read_at(&mut file, end - 128, &mut marker)?;
        if &marker == b"TAG" {
            end -= 128;
        }
    }
    if end >= 32 {
        let mut footer = [0; 32];
        read_at(&mut file, end - 32, &mut footer)?;
        if &footer[..8] == b"APETAGEX" {
            let size = u32::from_le_bytes(footer[12..16].try_into().unwrap()) as u64;
            if size < 32 || size > end {
                return Err("invalid APE tag length".into());
            }
            end -= size;
            // APEv2's optional header is not included in the tag size.
            if end >= 32 {
                let mut header = [0; 8];
                read_at(&mut file, end - 32, &mut header)?;
                if &header == b"APETAGEX" {
                    end -= 32;
                }
            }
        }
    }
    let mut position = 0;
    let mut count = 0;
    let mut start = 0;
    while position < end {
        if end - position >= 10 {
            let mut tag = [0; 10];
            read_at(&mut file, position, &mut tag)?;
            if &tag[..3] == b"ID3" {
                if !matches!(tag[3], 2..=4) || tag[6..10].iter().any(|b| b & 0x80 != 0) {
                    return Err("invalid ID3 tag header".into());
                }
                let size = tag[6..10]
                    .iter()
                    .fold(0u64, |size, byte| (size << 7) | u64::from(*byte));
                let footer = if tag[3] == 4 && tag[5] & 0x10 != 0 {
                    10
                } else {
                    0
                };
                position += 10 + size + footer;
                if position > end {
                    return Err("truncated ID3 tag".into());
                }
                continue;
            }
        }
        let mut header = [0; 7];
        let needed = if codec == "mp3" { 4 } else { 7 };
        if end - position < needed as u64 {
            return Err("truncated audio frame header".into());
        }
        read_at(&mut file, position, &mut header[..needed])?;
        let length = if codec == "mp3" {
            mp3_frame_length(&header)?
        } else {
            aac_frame_length(&header)?
        };
        if position + length > end {
            return Err("truncated audio frame payload".into());
        }
        if count == 0 {
            start = position;
        }
        position += length;
        count += 1;
    }
    if count == 0 {
        return Err("no audio frames".into());
    }
    Ok((start, end))
}

fn mp3_frame_length(h: &[u8]) -> Result<u64, String> {
    let version = (h[1] >> 3) & 3;
    let layer = (h[1] >> 1) & 3;
    let bitrate_index = (h[2] >> 4) as usize;
    let rate_index = ((h[2] >> 2) & 3) as usize;
    if h[0] != 0xff
        || h[1] & 0xe0 != 0xe0
        || version == 1
        || layer != 1
        || bitrate_index == 0
        || bitrate_index == 15
        || rate_index == 3
    {
        return Err("invalid or unsupported MP3 frame header".into());
    }
    let bitrates = if version == 3 {
        [
            0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320,
        ]
    } else {
        [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160]
    };
    let divisor = match version {
        3 => 1,
        2 => 2,
        _ => 4,
    };
    let rate = [44100, 48000, 32000][rate_index] / divisor;
    let coefficient = if version == 3 { 144_000 } else { 72_000 };
    Ok(coefficient * bitrates[bitrate_index] / rate + u64::from((h[2] >> 1) & 1))
}

fn aac_frame_length(h: &[u8]) -> Result<u64, String> {
    if h[0] != 0xff || h[1] & 0xf6 != 0xf0 || (h[2] >> 2) & 15 >= 13 {
        return Err("invalid AAC ADTS frame header".into());
    }
    let length = (u64::from(h[3] & 3) << 11) | (u64::from(h[4]) << 3) | u64::from(h[5] >> 5);
    let header_length = if h[1] & 1 == 0 { 9 } else { 7 };
    if length <= header_length {
        return Err("empty or invalid AAC frame".into());
    }
    Ok(length)
}

// Expose only the verified audio portion to the demuxer, without modifying the
// original file (which is archived byte for byte, including trailing metadata).
struct AudioFile {
    file: File,
    start: u64,
    len: u64,
}

impl Read for AudioFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let remaining = self
            .len
            .saturating_sub(self.file.stream_position()?.saturating_sub(self.start));
        let count = remaining.min(buf.len() as u64) as usize;
        self.file.read(&mut buf[..count])
    }
}

impl Seek for AudioFile {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        let position = match position {
            SeekFrom::Start(offset) => i128::from(offset),
            SeekFrom::End(offset) => i128::from(self.len) + i128::from(offset),
            SeekFrom::Current(offset) => {
                i128::from(self.file.stream_position()?) - i128::from(self.start)
                    + i128::from(offset)
            }
        };
        let physical = position + i128::from(self.start);
        if position < 0 || physical > i128::from(u64::MAX) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid seek",
            ));
        }
        self.file.seek(SeekFrom::Start(physical as u64))?;
        Ok(position as u64)
    }
}

impl MediaSource for AudioFile {
    fn is_seekable(&self) -> bool {
        true
    }
    fn byte_len(&self) -> Option<u64> {
        Some(self.len)
    }
}

fn flac_audio_end(path: &Path) -> Result<Option<u64>, Error> {
    let mut file = File::open(path)?;
    let len = file.metadata()?.len();
    // A FLAC frame cannot exceed 16 MiB. Keep the recovery scan bounded, and
    // only accommodate a small trailer. Larger malformed tails remain errors.
    let start = len.saturating_sub((1 << 24) + 128);
    file.seek(SeekFrom::Start(start))?;
    let mut tail = Vec::new();
    file.read_to_end(&mut tail)?;
    let mut candidates = 0;
    for index in (0..tail.len().saturating_sub(2)).rev() {
        if tail[index] != 0xff || tail[index + 1] & 0xfe != 0xf8 {
            continue;
        }
        candidates += 1;
        if candidates > 32 {
            break;
        }
        let mut crc = 0u16;
        for (offset, byte) in tail[index..].iter().enumerate() {
            crc ^= u16::from(*byte) << 8;
            for _ in 0..8 {
                crc = if crc & 0x8000 != 0 {
                    (crc << 1) ^ 0x8005
                } else {
                    crc << 1
                };
            }
            let end = index + offset + 1;
            let trailer = tail.len() - end;
            if crc == 0 && offset >= 7 && (1..=128).contains(&trailer) {
                return Ok(Some(start + end as u64));
            }
        }
    }
    Ok(None)
}
