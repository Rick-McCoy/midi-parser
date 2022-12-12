use encoding_rs::WINDOWS_1252;
use nom::{
    bytes::complete::{tag, take},
    number::complete::be_u8,
    IResult,
};

use crate::variable_length_quantity::VariableLengthQuantity;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MetaEvent {
    SequenceNumber,
    TextEvent {
        length: VariableLengthQuantity,
        text: String,
    },
    CopyrightNotice {
        length: VariableLengthQuantity,
        text: String,
    },
    SequenceOrTrackName {
        length: VariableLengthQuantity,
        text: String,
    },
    InstrumentName {
        length: VariableLengthQuantity,
        text: String,
    },
    Lyric {
        length: VariableLengthQuantity,
        text: String,
    },
    Marker {
        length: VariableLengthQuantity,
        text: String,
    },
    CuePoint {
        length: VariableLengthQuantity,
        text: String,
    },
    MidiChannelPrefix {
        channel: u8,
    },
    EndOfTrack,
    SetTempo {
        tempo: u32,
    },
    SmpteOffset {
        hour: u8,
        minute: u8,
        second: u8,
        frame: u8,
        subframe: u8,
    },
    TimeSignature {
        numerator: u8,
        denominator: u8,
        clocks_per_metronome_click: u8,
        thirty_seconds_per_quarter_note: u8,
    },
    KeySignature {
        key: u8,
        scale: u8,
    },
    SequencerSpecificEvent {
        length: VariableLengthQuantity,
        data: Vec<u8>,
    },
    UnknownMetaEvent {
        meta_type: u8,
        length: VariableLengthQuantity,
        data: Vec<u8>,
    },
}

impl MetaEvent {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, _) = tag(&[0xff])(input)?;
        let (input, meta_type) = be_u8(input)?;
        match meta_type {
            0x00 => {
                let (input, _) = tag(&[0x02])(input)?;
                Ok((input, Self::SequenceNumber))
            }
            0x01 | 0x02 | 0x03 | 0x04 | 0x05 | 0x06 | 0x07 => {
                let (input, length) = VariableLengthQuantity::parse(input)?;
                let (input, text) = take(length.value as usize)(input)?;
                let (text, _, replacement_used) = WINDOWS_1252.decode(text);
                if replacement_used {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Verify,
                    )));
                }
                let text = text.to_string();
                Ok((
                    input,
                    match meta_type {
                        0x01 => Self::TextEvent { length, text },
                        0x02 => Self::CopyrightNotice { length, text },
                        0x03 => Self::SequenceOrTrackName { length, text },
                        0x04 => Self::InstrumentName { length, text },
                        0x05 => Self::Lyric { length, text },
                        0x06 => Self::Marker { length, text },
                        0x07 => Self::CuePoint { length, text },
                        _ => unreachable!(),
                    },
                ))
            }
            0x20 => {
                let (input, _) = tag(&[0x01])(input)?;
                let (input, channel) = be_u8(input)?;
                Ok((input, Self::MidiChannelPrefix { channel }))
            }
            0x2f => {
                let (input, _) = tag(&[0x00])(input)?;
                Ok((input, Self::EndOfTrack))
            }
            0x51 => {
                let (input, _) = tag(&[0x03])(input)?;
                let (input, tempo) = take(3usize)(input)?;
                let tempo = u32::from_be_bytes([0, tempo[0], tempo[1], tempo[2]]);
                Ok((input, Self::SetTempo { tempo }))
            }
            0x54 => {
                let (input, _) = tag(&[0x05])(input)?;
                let (input, data) = take(5usize)(input)?;
                Ok((
                    input,
                    Self::SmpteOffset {
                        hour: data[0],
                        minute: data[1],
                        second: data[2],
                        frame: data[3],
                        subframe: data[4],
                    },
                ))
            }
            0x58 => {
                let (input, _) = tag(&[0x04])(input)?;
                let (input, data) = take(4usize)(input)?;
                Ok((
                    input,
                    Self::TimeSignature {
                        numerator: data[0],
                        denominator: data[1],
                        clocks_per_metronome_click: data[2],
                        thirty_seconds_per_quarter_note: data[3],
                    },
                ))
            }
            0x59 => {
                let (input, _) = tag(&[0x02])(input)?;
                let (input, data) = take(2usize)(input)?;
                Ok((
                    input,
                    Self::KeySignature {
                        key: data[0],
                        scale: data[1],
                    },
                ))
            }
            0x7f => {
                let (input, length) = VariableLengthQuantity::parse(input)?;
                let (input, data) = take(length.value as usize)(input)?;
                Ok((
                    input,
                    Self::SequencerSpecificEvent {
                        length,
                        data: data.to_vec(),
                    },
                ))
            }
            _ => {
                let (input, length) = VariableLengthQuantity::parse(input)?;
                let (input, data) = take(length.value as usize)(input)?;
                Ok((
                    input,
                    Self::UnknownMetaEvent {
                        meta_type,
                        length,
                        data: data.to_vec(),
                    },
                ))
            }
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::SequenceNumber => vec![0xff, 0x00, 0x02],
            Self::TextEvent { length, text } => [
                &[0xff, 0x01],
                length.to_bytes().as_slice(),
                WINDOWS_1252.encode(text).0.as_ref(),
            ]
            .concat(),
            Self::CopyrightNotice { length, text } => [
                &[0xff, 0x02],
                length.to_bytes().as_slice(),
                WINDOWS_1252.encode(text).0.as_ref(),
            ]
            .concat(),
            Self::SequenceOrTrackName { length, text } => [
                &[0xff, 0x03],
                length.to_bytes().as_slice(),
                WINDOWS_1252.encode(text).0.as_ref(),
            ]
            .concat(),
            Self::InstrumentName { length, text } => [
                &[0xff, 0x04],
                length.to_bytes().as_slice(),
                WINDOWS_1252.encode(text).0.as_ref(),
            ]
            .concat(),
            Self::Lyric { length, text } => [
                &[0xff, 0x05],
                length.to_bytes().as_slice(),
                WINDOWS_1252.encode(text).0.as_ref(),
            ]
            .concat(),
            Self::Marker { length, text } => [
                &[0xff, 0x06],
                length.to_bytes().as_slice(),
                WINDOWS_1252.encode(text).0.as_ref(),
            ]
            .concat(),
            Self::CuePoint { length, text } => [
                &[0xff, 0x07],
                length.to_bytes().as_slice(),
                WINDOWS_1252.encode(text).0.as_ref(),
            ]
            .concat(),
            Self::MidiChannelPrefix { channel } => vec![0xff, 0x20, 0x01, *channel],
            Self::EndOfTrack => vec![0xff, 0x2f, 0x00],
            Self::SetTempo { tempo } => {
                let tempo = tempo.to_be_bytes();
                vec![0xff, 0x51, 0x03, tempo[1], tempo[2], tempo[3]]
            }
            Self::SmpteOffset {
                hour,
                minute,
                second,
                frame,
                subframe,
            } => vec![0xff, 0x54, 0x05, *hour, *minute, *second, *frame, *subframe],

            Self::TimeSignature {
                numerator,
                denominator,
                clocks_per_metronome_click,
                thirty_seconds_per_quarter_note,
            } => {
                vec![
                    0xff,
                    0x58,
                    0x04,
                    *numerator,
                    *denominator,
                    *clocks_per_metronome_click,
                    *thirty_seconds_per_quarter_note,
                ]
            }
            Self::KeySignature { key, scale } => vec![0xff, 0x59, 0x02, *key, *scale],
            Self::SequencerSpecificEvent { length, data } => {
                [&[0xff, 0x7f], length.to_bytes().as_slice(), data].concat()
            }
            Self::UnknownMetaEvent {
                meta_type,
                length,
                data,
            } => [&[0xff, *meta_type], length.to_bytes().as_slice(), data].concat(),
        }
    }
}
