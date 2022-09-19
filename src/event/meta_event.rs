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
                let text = String::from_utf8(text.to_vec()).expect("Invalid UTF-8");
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
            0x2F => {
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
            0x7F => {
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
        let mut bytes = vec![0xff];
        match self {
            Self::SequenceNumber => {
                bytes.push(0x00);
                bytes.push(0x02);
            }
            Self::TextEvent { length, text } => {
                bytes.push(0x01);
                bytes.extend(length.to_bytes());
                bytes.extend(text.as_bytes());
            }
            Self::CopyrightNotice { length, text } => {
                bytes.push(0x02);
                bytes.extend(length.to_bytes());
                bytes.extend(text.as_bytes());
            }
            Self::SequenceOrTrackName { length, text } => {
                bytes.push(0x03);
                bytes.extend(length.to_bytes());
                bytes.extend(text.as_bytes());
            }
            Self::InstrumentName { length, text } => {
                bytes.push(0x04);
                bytes.extend(length.to_bytes());
                bytes.extend(text.as_bytes());
            }
            Self::Lyric { length, text } => {
                bytes.push(0x05);
                bytes.extend(length.to_bytes());
                bytes.extend(text.as_bytes());
            }
            Self::Marker { length, text } => {
                bytes.push(0x06);
                bytes.extend(length.to_bytes());
                bytes.extend(text.as_bytes());
            }
            Self::CuePoint { length, text } => {
                bytes.push(0x07);
                bytes.extend(length.to_bytes());
                bytes.extend(text.as_bytes());
            }
            Self::MidiChannelPrefix { channel } => {
                bytes.push(0x20);
                bytes.push(0x01);
                bytes.push(*channel);
            }
            Self::EndOfTrack => {
                bytes.push(0x2F);
                bytes.push(0x00);
            }
            Self::SetTempo { tempo } => {
                bytes.push(0x51);
                bytes.push(0x03);
                bytes.extend(&tempo.to_be_bytes()[1..]);
            }
            Self::SmpteOffset {
                hour,
                minute,
                second,
                frame,
                subframe,
            } => {
                bytes.push(0x54);
                bytes.push(0x05);
                bytes.push(*hour);
                bytes.push(*minute);
                bytes.push(*second);
                bytes.push(*frame);
                bytes.push(*subframe);
            }
            Self::TimeSignature {
                numerator,
                denominator,
                clocks_per_metronome_click,
                thirty_seconds_per_quarter_note,
            } => {
                bytes.push(0x58);
                bytes.push(0x04);
                bytes.push(*numerator);
                bytes.push(*denominator);
                bytes.push(*clocks_per_metronome_click);
                bytes.push(*thirty_seconds_per_quarter_note);
            }
            Self::KeySignature { key, scale } => {
                bytes.push(0x59);
                bytes.push(0x02);
                bytes.push(*key);
                bytes.push(*scale);
            }
            Self::SequencerSpecificEvent { length, data } => {
                bytes.push(0x7F);
                bytes.extend(length.to_bytes());
                bytes.extend(data);
            }
            Self::UnknownMetaEvent {
                meta_type,
                length,
                data,
            } => {
                bytes.push(*meta_type);
                bytes.extend(length.to_bytes());
                bytes.extend(data);
            }
        }
        bytes
    }
}
