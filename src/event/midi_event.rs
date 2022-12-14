use nom::{combinator::peek, IResult};

use crate::utils::be_u7;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ChannelVoiceMessage {
    NoteOff {
        channel: u8,
        note: u8,
        velocity: u8,
    },
    NoteOn {
        channel: u8,
        note: u8,
        velocity: u8,
    },
    PolyphonicKeyPressure {
        channel: u8,
        note: u8,
        pressure: u8,
    },
    ControlChange {
        channel: u8,
        controller: u8,
        value: u8,
    },
    ProgramChange {
        channel: u8,
        program: u8,
    },
    ChannelPressure {
        channel: u8,
        pressure: u8,
    },
    PitchBendChange {
        channel: u8,
        value: u16,
    },
}

impl ChannelVoiceMessage {
    pub fn parse(input: &[u8], status: u8) -> IResult<&[u8], Self> {
        let message_type = status >> 4;
        let channel = status & 0x0f;
        match message_type {
            0x8 => {
                let (input, note) = be_u7(input)?;
                let (input, velocity) = be_u7(input)?;
                Ok((
                    input,
                    Self::NoteOff {
                        channel,
                        note,
                        velocity,
                    },
                ))
            }
            0x9 => {
                let (input, note) = be_u7(input)?;
                let (input, velocity) = be_u7(input)?;
                Ok((
                    input,
                    Self::NoteOn {
                        channel,
                        note,
                        velocity,
                    },
                ))
            }
            0xa => {
                let (input, note) = be_u7(input)?;
                let (input, pressure) = be_u7(input)?;
                Ok((
                    input,
                    Self::PolyphonicKeyPressure {
                        channel,
                        note,
                        pressure,
                    },
                ))
            }
            0xb => {
                let (input, controller) = be_u7(input)?;
                let (input, value) = be_u7(input)?;
                Ok((
                    input,
                    Self::ControlChange {
                        channel,
                        controller,
                        value,
                    },
                ))
            }
            0xc => {
                let (input, program) = be_u7(input)?;
                Ok((input, Self::ProgramChange { channel, program }))
            }
            0xd => {
                let (input, pressure) = be_u7(input)?;
                Ok((input, Self::ChannelPressure { channel, pressure }))
            }
            0xe => {
                let (input, lsb) = be_u7(input)?;
                let (input, msb) = be_u7(input)?;
                let value = ((msb as u16) << 7) | (lsb as u16);
                Ok((input, Self::PitchBendChange { channel, value }))
            }
            _ => unreachable!(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::NoteOff { note, velocity, .. } | Self::NoteOn { note, velocity, .. } => {
                vec![self.get_status(), *note, *velocity]
            }
            Self::PolyphonicKeyPressure { note, pressure, .. } => {
                vec![self.get_status(), *note, *pressure]
            }
            Self::ControlChange {
                controller, value, ..
            } => {
                vec![self.get_status(), *controller, *value]
            }
            Self::ProgramChange { program, .. } => {
                vec![self.get_status(), *program]
            }
            Self::ChannelPressure { pressure, .. } => {
                vec![self.get_status(), *pressure]
            }
            Self::PitchBendChange { value, .. } => {
                vec![self.get_status(), (value & 0x7f) as u8, (value >> 7) as u8]
            }
        }
    }

    pub fn get_status(&self) -> u8 {
        match self {
            Self::NoteOff { channel, .. } => 0x80 | channel,
            Self::NoteOn { channel, .. } => 0x90 | channel,
            Self::PolyphonicKeyPressure { channel, .. } => 0xa0 | channel,
            Self::ControlChange { channel, .. } => 0xb0 | channel,
            Self::ProgramChange { channel, .. } => 0xc0 | channel,
            Self::ChannelPressure { channel, .. } => 0xd0 | channel,
            Self::PitchBendChange { channel, .. } => 0xe0 | channel,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ModeMessage {
    LocalControlOff,
    LocalControlOn,
    AllNotesOff,
    OmniModeOff,
    OmniModeOn,
    MonoModeOn { n: u8 },
    PolyModeOn,
}

impl ModeMessage {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, controller) = be_u7(input)?;
        let (input, value) = be_u7(input)?;
        match controller {
            0x7a => match value {
                0x00 => Ok((input, Self::LocalControlOff)),
                0x7f => Ok((input, Self::LocalControlOn)),
                _ => panic!("Invalid value for local control {}", value),
            },
            0x7b => Ok((input, Self::AllNotesOff)),
            0x7c => Ok((input, Self::OmniModeOff)),
            0x7d => Ok((input, Self::OmniModeOn)),
            0x7e => Ok((input, Self::MonoModeOn { n: value })),
            0x7f => Ok((input, Self::PolyModeOn)),
            _ => panic!("Invalid controller for mode message {}", controller),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::LocalControlOff => vec![0x7f, 0x00],
            Self::LocalControlOn => vec![0x7f, 0x7f],
            Self::AllNotesOff => vec![0x7b, 0x00],
            Self::OmniModeOff => vec![0x7c, 0x00],
            Self::OmniModeOn => vec![0x7d, 0x00],
            Self::MonoModeOn { n } => vec![0x7e, *n],
            Self::PolyModeOn => vec![0x7f, 0x00],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ChannelModeMessage {
    channel: u8,
    message: ModeMessage,
}

impl ChannelModeMessage {
    pub fn parse(input: &[u8], status: u8) -> IResult<&[u8], Self> {
        let message_type = status >> 4;
        assert_eq!(message_type, 0xb);
        let channel = status & 0x0f;
        let (input, message) = ModeMessage::parse(input)?;
        Ok((input, Self { channel, message }))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        [vec![self.get_status()], self.message.to_bytes()].concat()
    }

    pub fn get_status(&self) -> u8 {
        0xb0 | self.channel
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ChannelMessage {
    ChannelVoiceMessage(ChannelVoiceMessage),
    ChannelModeMessage(ChannelModeMessage),
}

impl ChannelMessage {
    pub fn parse(input: &[u8], status: u8) -> IResult<&[u8], Self> {
        let message_type = status >> 4;
        match message_type {
            0x8 | 0x9 | 0xa | 0xc | 0xd | 0xe => {
                let (input, message) = ChannelVoiceMessage::parse(input, status)?;
                Ok((input, Self::ChannelVoiceMessage(message)))
            }
            0xb => {
                let (input, controller) = peek(be_u7)(input)?;
                match controller {
                    0x7a | 0x7b | 0x7c | 0x7d | 0x7e | 0x7f => {
                        let (input, message) = ChannelModeMessage::parse(input, status)?;
                        Ok((input, Self::ChannelModeMessage(message)))
                    }
                    _ => {
                        let (input, message) = ChannelVoiceMessage::parse(input, status)?;
                        Ok((input, Self::ChannelVoiceMessage(message)))
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::ChannelVoiceMessage(message) => message.to_bytes(),
            Self::ChannelModeMessage(message) => message.to_bytes(),
        }
    }

    pub fn get_status(&self) -> u8 {
        match self {
            Self::ChannelVoiceMessage(message) => message.get_status(),
            Self::ChannelModeMessage(message) => message.get_status(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SystemCommonMessage {
    SongPositionPointer { value: u16 },
    SongSelect { song: u8 },
    TuneRequest,
    EndOfExclusive,
}

impl SystemCommonMessage {
    pub fn parse(input: &[u8], status: u8) -> IResult<&[u8], Self> {
        assert_eq!(status >> 4, 0xf);
        let message_type = status & 0x0f;
        match message_type {
            0x2 => {
                let (input, lsb) = be_u7(input)?;
                let (input, msb) = be_u7(input)?;
                let value = ((msb as u16) << 7) | (lsb as u16);
                Ok((input, Self::SongPositionPointer { value }))
            }
            0x3 => {
                let (input, song) = be_u7(input)?;
                Ok((input, Self::SongSelect { song }))
            }
            0x6 => Ok((input, Self::TuneRequest)),
            0x7 => Ok((input, Self::EndOfExclusive)),
            _ => unreachable!(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::SongPositionPointer { value } => {
                vec![0xf2, (value & 0x7f) as u8, ((value >> 7) & 0x7f) as u8]
            }
            Self::SongSelect { song } => vec![0xf3, *song],
            Self::TuneRequest => vec![0xf6],
            Self::EndOfExclusive => vec![0xf7],
        }
    }

    pub fn get_status(&self) -> u8 {
        match self {
            Self::SongPositionPointer { .. } => 0xf2,
            Self::SongSelect { .. } => 0xf3,
            Self::TuneRequest => 0xf6,
            Self::EndOfExclusive => 0xf7,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SystemRealTimeMessage {
    TimingClock,
    Start,
    Continue,
    Stop,
    ActiveSensing,
    SystemReset,
}

impl SystemRealTimeMessage {
    pub fn parse(input: &[u8], status: u8) -> IResult<&[u8], Self> {
        assert_eq!(status >> 4, 0xf);
        let message_type = status & 0x0f;
        match message_type {
            0x8 => Ok((input, Self::TimingClock)),
            0xa => Ok((input, Self::Start)),
            0xb => Ok((input, Self::Continue)),
            0xc => Ok((input, Self::Stop)),
            0xe => Ok((input, Self::ActiveSensing)),
            0xf => Ok((input, Self::SystemReset)),
            _ => unreachable!(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::TimingClock => vec![0xf8],
            Self::Start => vec![0xfa],
            Self::Continue => vec![0xfb],
            Self::Stop => vec![0xfc],
            Self::ActiveSensing => vec![0xfe],
            Self::SystemReset => vec![0xff],
        }
    }

    pub fn get_status(&self) -> u8 {
        match self {
            Self::TimingClock => 0xf8,
            Self::Start => 0xfa,
            Self::Continue => 0xfb,
            Self::Stop => 0xfc,
            Self::ActiveSensing => 0xfe,
            Self::SystemReset => 0xff,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SystemMessage {
    SystemCommonMessage(SystemCommonMessage),
    SystemRealTimeMessage(SystemRealTimeMessage),
}

impl SystemMessage {
    pub fn parse(input: &[u8], status: u8) -> IResult<&[u8], Self> {
        assert_eq!(status >> 4, 0xf);
        let message_type = status & 0x0f;
        match message_type {
            0x2 | 0x3 | 0x6 | 0x7 => {
                let (input, message) = SystemCommonMessage::parse(input, status)?;
                Ok((input, Self::SystemCommonMessage(message)))
            }
            0x8 | 0xa | 0xb | 0xc | 0xe | 0xf => {
                let (input, message) = SystemRealTimeMessage::parse(input, status)?;
                Ok((input, Self::SystemRealTimeMessage(message)))
            }
            _ => unreachable!(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::SystemCommonMessage(message) => message.to_bytes(),
            Self::SystemRealTimeMessage(message) => message.to_bytes(),
        }
    }

    pub fn get_status(&self) -> u8 {
        match self {
            Self::SystemCommonMessage(message) => message.get_status(),
            Self::SystemRealTimeMessage(message) => message.get_status(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MidiMessage {
    ChannelMessage(ChannelMessage),
    SystemMessage(SystemMessage),
}

impl MidiMessage {
    pub fn parse(input: &[u8], status: u8) -> IResult<&[u8], Self> {
        let message_type = status >> 4;
        match message_type {
            0x8 | 0x9 | 0xa | 0xb | 0xc | 0xd | 0xe => {
                let (input, message) = ChannelMessage::parse(input, status)?;
                Ok((input, Self::ChannelMessage(message)))
            }
            0xf => {
                let (input, message) = SystemMessage::parse(input, status)?;
                Ok((input, Self::SystemMessage(message)))
            }
            _ => panic!("Invalid message type {}", message_type),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::ChannelMessage(message) => message.to_bytes(),
            Self::SystemMessage(message) => message.to_bytes(),
        }
    }

    pub fn get_status(&self) -> u8 {
        match self {
            Self::ChannelMessage(message) => message.get_status(),
            Self::SystemMessage(message) => message.get_status(),
        }
    }
}
