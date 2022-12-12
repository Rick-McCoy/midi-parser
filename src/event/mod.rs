use self::{meta_event::MetaEvent, midi_event::MidiMessage, sysex_event::SysExEvent};
use crate::variable_length_quantity::VariableLengthQuantity;
use nom::{combinator::peek, number::complete::be_u8, IResult};

pub mod meta_event;
pub mod midi_event;
pub mod sysex_event;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Event {
    MidiEvent(MidiMessage),
    SysExEvent(SysExEvent),
    MetaEvent(MetaEvent),
}

impl Event {
    pub fn parse(input: &[u8], running_status: u8) -> IResult<&[u8], Self> {
        let (input, status) = peek(be_u8)(input)?;
        match status {
            0xf0 | 0xf7 => {
                let (input, event) = SysExEvent::parse(input)?;
                Ok((input, Self::SysExEvent(event)))
            }
            0xff => {
                let (input, event) = MetaEvent::parse(input)?;
                Ok((input, Self::MetaEvent(event)))
            }
            _ => {
                if status >> 7 == 0 {
                    let (input, event) = MidiMessage::parse(input, running_status)?;
                    Ok((input, Self::MidiEvent(event)))
                } else {
                    let (input, status) = be_u8(input)?;
                    let (input, event) = MidiMessage::parse(input, status)?;
                    Ok((input, Self::MidiEvent(event)))
                }
            }
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::MidiEvent(event) => event.to_bytes(),
            Self::SysExEvent(event) => event.to_bytes(),
            Self::MetaEvent(event) => event.to_bytes(),
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

    pub fn to_bytes(&self) -> Vec<u8> {
        [self.delta_time.to_bytes(), self.event.to_bytes()].concat()
    }

    pub fn get_status(&self) -> u8 {
        self.event.get_status()
    }
}
