use crate::{
    meta_event::MetaEvent, midi_event::MidiMessage, sysex_event::SysExEvent,
    variable_length_quantity::VariableLengthQuantity,
};
use nom::{combinator::peek, number::complete::be_u8, IResult};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Event {
    MidiEvent(MidiMessage),
    SysExEvent(SysExEvent),
    MetaEvent(MetaEvent),
}

impl Event {
    pub fn parse(input: &[u8], running_status: u8) -> IResult<&[u8], Self> {
        let (input, status) = peek(be_u8)(input)?;
        if status & 0x80 != 0x00 {
            match status {
                0xF0 | 0xF7 => {
                    let (input, event) = SysExEvent::parse(input)?;
                    Ok((input, Self::SysExEvent(event)))
                }
                0xFF => {
                    let (input, event) = MetaEvent::parse(input)?;
                    Ok((input, Self::MetaEvent(event)))
                }
                _ => {
                    let (input, event) = MidiMessage::parse(input)?;
                    Ok((input, Self::MidiEvent(event)))
                }
            }
        } else {
            let (input, event) = MidiMessage::parse_with_running_status(input, running_status)?;
            Ok((input, Self::MidiEvent(event)))
        }
    }

    pub fn get_status(&self) -> u8 {
        match self {
            Self::MidiEvent(midi_message) => midi_message.get_status(),
            Self::SysExEvent(sysex_event) => sysex_event.get_status(),
            Self::MetaEvent(_) => 0xff,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MTrkEvent {
    pub delta_time: VariableLengthQuantity,
    pub event: Event,
}

impl MTrkEvent {
    pub fn parse(input: &[u8], running_status: u8) -> IResult<&[u8], Self> {
        let (input, delta_time) = VariableLengthQuantity::parse(input)?;
        let (input, event) = Event::parse(input, running_status)?;
        Ok((input, Self { delta_time, event }))
    }
}
