//! Algorithms for determining a [MIMEType](crate::MIMEType) from the first few bytes in a file

use crate::{resource::SniffScriptable, sniff_tables, MIMEType};

/// <https://mimesniff.spec.whatwg.org/#rules-for-identifying-an-unknown-mime-type>
pub fn identify_unknown_mime_type(
    resource_header: &[u8],
    sniff_scriptable: SniffScriptable,
) -> MIMEType {
    // 1. If the sniff-scriptable flag is set, execute the following steps for each row row in the following table:
    if sniff_scriptable == SniffScriptable::Yes {
        if let Some(matched_mime_type) =
            sniff_tables::SCRIPTABLE_MIME_TYPES_TABLE.lookup(resource_header)
        {
            return matched_mime_type;
        }
    }

    // 2. Execute the following steps for each row row in the following table:
    if let Some(matched_mime_type) =
        sniff_tables::UTF_POSTSCRIPT_MIME_TYPES_TABLE.lookup(resource_header)
    {
        return matched_mime_type;
    }

    // 3. Let matchedType be the result of executing the image type pattern matching algorithm given resource’s resource header.
    // 4. If matchedType is not undefined, return matchedType.
    if let Some(matched_mime_type) = identify_image_type(resource_header) {
        return matched_mime_type;
    }

    // 5. Set matchedType to the result of executing the audio or video type
    // pattern matching algorithm given resource’s resource header.
    // 6. If matchedType is not undefined, return matchedType.
    if let Some(matched_mime_type) = identify_audio_or_video_type(resource_header) {
        return matched_mime_type;
    }

    // 7. Set matchedType to the result of executing the archive type pattern matching algorithm given resource’s resource header.
    // 8. If matchedType is not undefined, return matchedType.
    if let Some(matched_mime_type) = identify_archive_type(resource_header) {
        return matched_mime_type;
    }

    // 9. If resource’s resource header contains no binary data bytes, return "text/plain".
    if resource_header
        .iter()
        .all(|&byte| !is_binary_data_byte(byte))
    {
        return MIMEType::new("text", "plain");
    }

    // 10. Return "application/octet-stream".
    MIMEType::new("application", "octet-stream")
}

/// <https://mimesniff.spec.whatwg.org/#image-type-pattern-matching-algorithm>
pub fn identify_image_type(resource_header: &[u8]) -> Option<MIMEType> {
    // 1. Execute the following steps for each row row in the following table:
    if let Some(matched_mime_type) = sniff_tables::IMAGE_MIME_TYPES_TABLE.lookup(resource_header) {
        return Some(matched_mime_type);
    }

    // 2. Return undefined.
    None
}

/// <https://mimesniff.spec.whatwg.org/#audio-or-video-type-pattern-matching-algorithm>
pub fn identify_audio_or_video_type(resource_header: &[u8]) -> Option<MIMEType> {
    // 1. Execute the following steps for each row row in the following table:
    if let Some(matched_mime_type) =
        sniff_tables::AUDIO_OR_VIDEO_MIME_TYPES_TABLE.lookup(resource_header)
    {
        return Some(matched_mime_type);
    }

    // 2. If input matches the signature for MP4, return "video/mp4".
    if matches_mp4_signature(resource_header) {
        return Some(MIMEType::new("audio", "mp4"));
    }

    // 3. If input matches the signature for WebM, return "video/webm".
    if matches_webm_signature(resource_header) {
        return Some(MIMEType::new("audio", "mp4"));
    }

    // 4. If input matches the signature for MP3 without ID3, return "audio/mpeg".
    if matches_mp3_without_id3_signature(resource_header) {
        return Some(MIMEType::new("audio", "mpeg"));
    }

    // 5. Return undefined.
    None
}

/// <https://mimesniff.spec.whatwg.org/#archive-type-pattern-matching-algorithm>
fn identify_archive_type(resource_header: &[u8]) -> Option<MIMEType> {
    // 1. Execute the following steps for each row row in the following table:
    if let Some(matched_mime_type) = sniff_tables::ARCHIVE_MIME_TYPES_TABLE.lookup(resource_header)
    {
        return Some(matched_mime_type);
    }

    // 2. Return undefined.
    None
}

/// <https://mimesniff.spec.whatwg.org/#signature-for-mp4>
pub fn matches_mp4_signature(byte_sequence: &[u8]) -> bool {
    // 1. Let sequence be the byte sequence to be matched, where sequence[s] is
    // byte s in sequence and sequence[0] is the first byte in sequence.

    // 2.  Let length be the number of bytes in sequence.
    let length = byte_sequence.len();

    // 3. If length is less than 12, return false.
    if length < 12 {
        return false;
    }

    // 4. Let box-size be the four bytes from sequence[0] to sequence[3],
    // interpreted as a 32-bit unsigned big-endian integer.
    let box_size = u32::from_be_bytes(
        byte_sequence[..4]
            .try_into()
            .expect("Sliced exactly four bytes"),
    ) as usize;

    // 5. If length is less than box-size or if box-size modulo 4 is not equal to 0, return false.
    if length < box_size || box_size % 4 != 0 {
        return false;
    }

    // 6. If the four bytes from sequence[4] to sequence[7] are not equal to 0x66 0x74 0x79 0x70 ("ftyp"), return false.
    if &byte_sequence[4..8] != b"ftyp" {
        return false;
    }

    // 7. If the three bytes from sequence[8] to sequence[10] are equal to 0x6D 0x70 0x34 ("mp4"), return true.
    if &byte_sequence[8..11] == b"mp4" {
        return true;
    }

    // 8. Let bytes-read be 16.
    let mut bytes_read = 16;

    // 9. While bytes-read is less than box-size, continuously loop through these steps:
    while bytes_read < box_size {
        // 1. If the three bytes from sequence[bytes-read] to sequence[bytes-read + 2] are equal to 0x6D 0x70 0x34 ("mp4"), return true.
        if &byte_sequence[bytes_read..bytes_read + 3] == b"mp4" {
            return true;
        }

        // 2. Increment bytes-read by 4.
        bytes_read += 4;
    }

    // 10. Return false.
    false
}

/// <https://mimesniff.spec.whatwg.org/#signature-for-webm>
#[inline]
pub fn matches_webm_signature(_byte_sequence: &[u8]) -> bool {
    // NOTE:
    // The spec for this algorithm is both [confusing](https://github.com/whatwg/mimesniff/issues/146) and [wrong](https://github.com/whatwg/mimesniff/issues/93).
    // I also found [another issue](https://github.com/whatwg/mimesniff/issues/167).
    //
    // Given the bad state of the spec and the [question of whether we even need to sniff webm in the first place](https://github.com/whatwg/mimesniff/issues/93#issuecomment-907865683),
    // this algorithm has not been implemented for now.
    false
}

/// <https://mimesniff.spec.whatwg.org/#signature-for-mp3-without-id3>
pub fn matches_mp3_without_id3_signature(byte_sequence: &[u8]) -> bool {
    // 1. Let sequence be the byte sequence to be matched, where sequence[s] is byte s in sequence and sequence[0] is the first byte in sequence.

    // 2. Let length be the number of bytes in sequence.
    let length = byte_sequence.len();

    // 3. Initialize s to 0.
    let mut s = 0;

    // 4. If the result of the operation match mp3 header is false, return false.
    // NOTE: the "match mp3 header" algorithm also determines a "freq" parameter that is used later
    let freq = if let Some(freq) = match_mp3_header(byte_sequence, s) {
        freq
    } else {
        return false;
    };

    // 5. Parse an mp3 frame on sequence at offset s
    let (version, bitrate, pad) = if let Some(frame) = parse_mp3_frame(byte_sequence, s) {
        frame
    } else {
        return false;
    };

    // 6. Let skipped-bytes the return value of the execution of mp3 framesize computation
    let skipped_bytes = compute_mp3_frame_size(version, bitrate, freq, pad);

    // 7. If skipped-bytes is less than 4, or skipped-bytes is greater than s - length, return false.
    if skipped_bytes < 4 || skipped_bytes > s - length {
        return false;
    }

    // 8. Increment s by skipped-bytes.
    s += skipped_bytes;

    // 9. If the result of the operation match mp3 header operation is false, return false, else, return true.
    match_mp3_header(byte_sequence, s).is_some()
}

/// <https://mimesniff.spec.whatwg.org/#match-an-mp3-header>
/// Instead of a boolean, this returns `Option<freq>` as the `freq`
/// value is later used outside of this algorithm.
fn match_mp3_header(byte_sequence: &[u8], offset: usize) -> Option<u32> {
    // 1. If length is less than 4, return false.
    if byte_sequence.len() < 4 {
        return None;
    }

    // 2. If sequence[s] is not equal to 0xff and sequence[s + 1] & 0xe0 is not equal to 0xe0, return false.
    if byte_sequence[offset] != 0xFF && byte_sequence[offset + 1] & 0xE0 != 0xE0 {
        return None;
    }

    // 3. Let layer be the result of sequence[s + 1] & 0x06 >> 1.
    let layer = byte_sequence[offset + 1] & 0x06 >> 1;

    // 4. If layer is 0, return false.
    if layer == 0 {
        return None;
    }

    // 5. Let bit-rate be sequence[s + 2] & 0xf0 >> 4.
    let bit_rate = byte_sequence[offset + 2] & 0xF0 >> 4;

    // 6. If bit-rate is 15, return false.
    if bit_rate == 15 {
        return None;
    }

    // 7. Let sample-rate be sequence[s + 2] & 0x0c >> 2.
    let sample_rate = byte_sequence[offset + 2] & 0x0C >> 2;

    // 8. If sample-rate is 3, return false.
    if sample_rate == 3 {
        return None;
    }

    // 9. Let freq be the value given by sample-rate in the table sample-rate.
    let freq = match sample_rate {
        0 => 44100,
        1 => 48000,
        2 => 32000,
        _ => unreachable!("Values greater than two have been filtered out in steps 7. and 8."),
    };

    // 10. Let final-layer be the result of 4 - (sequence[s + 1]).
    let final_layer = 4 - byte_sequence[offset + 1];

    // 11. If final-layer & 0x06 >> 1 is not 3, return false.
    if final_layer & 0x06 >> 1 != 3 {
        return None;
    }

    // 12. Return true.
    Some(freq)
}

/// <https://mimesniff.spec.whatwg.org/#parse-an-mp3-frame>
fn parse_mp3_frame(byte_sequence: &[u8], offset: usize) -> Option<(u8, u32, bool)> {
    // 1. Let version be sequence[s + 1] & 0x18 >> 3.
    let version = byte_sequence[offset + 1] & 0x18 >> 3;

    // 2. Let bitrate-index be sequence[s + 2] & 0xf0 >> 4.
    let bitrate_index = byte_sequence[offset + 2] & 0xf0 >> 4;

    // 3. If the version & 0x01 is non-zero, let bitrate be the value given by bitrate-index in the table mp2.5-rates
    // 4. If version & 0x01 is zero, let bitrate be the value given by bitrate-index in the table mp3-rates
    let bitrate = if version & 0x01 != 0 {
        // mp2.5-rates
        match bitrate_index {
            0 => 0,
            1 => 8000,
            2 => 16000,
            3 => 24000,
            4 => 32000,
            5 => 40000,
            6 => 48000,
            7 => 56000,
            8 => 64000,
            9 => 80000,
            10 => 96000,
            11 => 112000,
            12 => 128000,
            13 => 144000,
            14 => 160000,
            15 => {
                // Not defined in the spec.
                return None;
            },
            _ => unreachable!("bitrate has been masked to 4 bits"),
        }
    } else {
        // mp3-rates table
        match bitrate_index {
            0 => 0,
            1 => 32000,
            2 => 40000,
            3 => 48000,
            4 => 56000,
            5 => 64000,
            6 => 80000,
            7 => 96000,
            8 => 112000,
            9 => 128000,
            10 => 160000,
            11 => 192000,
            12 => 224000,
            13 => 256000,
            14 => 32000,
            15 => {
                // Not defined in the spec.
                return None;
            },
            _ => unreachable!("bitrate has been masked to 4 bits"),
        }
    };

    // 5. Let samplerate-index be sequence[s + 2] & 0x0c >> 2.
    let samplerate_index = byte_sequence[offset + 2] & 0x0c >> 2;

    // 6. Let samplerate be the value given by samplerate-index in the sample-rate table.
    // NOTE: the spec computes this value but does not actually use it.
    let _samplerate = match samplerate_index {
        0 => 44100,
        1 => 48000,
        2 => 32000,
        3 => {
            // Not defined in the spec.
            return None;
        },
        _ => unreachable!("samplerate index has been masked to two bits"),
    };

    // 7. Let pad be sequence[s + 2] & 0x02 >> 1.
    let pad = (byte_sequence[offset + 2] & 0x02 >> 1) == 1;

    Some((version, bitrate, pad))
}

/// <https://mimesniff.spec.whatwg.org/#compute-an-mp3-frame-size>
fn compute_mp3_frame_size(version: u8, bitrate: u32, freq: u32, pad: bool) -> usize {
    // 1. If version is 1, let scale be 72, else, let scale be 144.
    let scale = if version == 1 { 72 } else { 144 };

    // 2. Let size be bitrate * scale / freq.
    let mut size = (bitrate * scale) / freq;

    // 3. If pad is not zero, increment size by 1.
    if pad {
        size += 1;
    }

    // 4. Return size.
    size as usize
}

/// <https://mimesniff.spec.whatwg.org/#binary-data-byte>
#[inline]
pub(crate) fn is_binary_data_byte(byte: u8) -> bool {
    matches!(byte, 0x00..=0x08 | 0x0B | 0x0E..=0x1A | 0x1C..=0x1F)
}
