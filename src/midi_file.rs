use crate::{header::HeaderChunk, track::TrackChunk};
use nom::{multi::count, IResult};

pub struct MidiFile {
    pub header: HeaderChunk,
    pub tracks: Vec<TrackChunk>,
}

impl MidiFile {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, header) = HeaderChunk::parse(input)?;
        let ntrks = header.data.ntrks as usize;
        let (input, tracks) = count(TrackChunk::parse, ntrks)(input)?;
        Ok((input, Self { header, tracks }))
    }
}

#[cfg(test)]
mod tests {
    use crate::event::{
        meta_event::{MetaEvent, MetaEventContent},
        midi_event::{ChannelMessage, ChannelVoiceMessage, MidiMessage},
        Event,
    };

    use super::MidiFile;

    #[test]
    fn test_parse() {
        let bytes = [
            0x4d, 0x54, 0x68, 0x64, // MThd
            0x00, 0x00, 0x00, 0x06, // header length
            0x00, 0x00, // format, 0
            0x00, 0x04, // ntrks, 4 tracks
            0x00, 0x60, // division, 96 ticks per quarter note
            0x4d, 0x54, 0x72, 0x6b, // MTrk
            0x00, 0x00, 0x00, 0x14, // chunk length (20 bytes)
            0x00, 0xff, 0x58, 0x04, 0x04, 0x02, 0x18, 0x08, // time signature
            0x00, 0xff, 0x51, 0x03, 0x07, 0xa1, 0x20, // tempo
            0x83, 0x00, 0xff, 0x2f, 0x00, // end of track
            0x4d, 0x54, 0x72, 0x6b, // MTrk
            0x00, 0x00, 0x00, 0x10, // chunk length (16 bytes)
            0x00, 0xc0, 0x05, // program change, channel 0, program 5
            0x81, 0x40, 0x90, 0x4c, 0x20, // note on, channel 0, note 76, velocity 32
            0x81, 0x40, 0x4c, 0x00, // note on, channel 0, note 76, velocity 0 (note off)
            0x00, 0xff, 0x2f, 0x00, // end of track
            0x4d, 0x54, 0x72, 0x6b, // MTrk
            0x00, 0x00, 0x00, 0x0f, // chunk length (15 bytes)
            0x00, 0xc1, 0x2e, // program change, channel 1, program 46
            0x60, 0x91, 0x43, 0x40, // note on, channel 1, note 67, velocity 64
            0x82, 0x20, 0x43, 0x00, // note on, channel 1, note 67, velocity 0 (note off)
            0x00, 0xff, 0x2f, 0x00, // end of track
            0x4d, 0x54, 0x72, 0x6b, // MTrk
            0x00, 0x00, 0x00, 0x15, // chunk length (21 bytes)
            0x00, 0xc2, 0x46, // program change, channel 2, program 70
            0x00, 0x92, 0x30, 0x60, // note on, channel 2, note 48, velocity 96
            0x00, 0x3c, 0x60, // note on, channel 2, note 60, velocity 96
            0x83, 0x00, 0x30, 0x00, // note on, channel 2, note 48, velocity 0 (note off)
            0x00, 0x3c, 0x00, // note on, channel 2, note 60, velocity 0 (note off)
            0x00, 0xff, 0x2f, 0x00, // end of track
        ];
        let (_, midi_file) = MidiFile::parse(&bytes).unwrap();
        assert_eq!(midi_file.header.chunk_type, "MThd");
        assert_eq!(midi_file.header.length, 6);
        assert_eq!(midi_file.header.data.format, 0);
        assert_eq!(midi_file.header.data.ntrks, 4);
        assert_eq!(midi_file.header.data.division, 96);
        assert_eq!(midi_file.tracks.len(), 4);
        assert_eq!(midi_file.tracks[0].chunk_type, "MTrk");
        assert_eq!(midi_file.tracks[0].length, 20);
        assert_eq!(midi_file.tracks[0].data.len(), 3);
        assert_eq!(midi_file.tracks[0].data[0].delta_time.value, 0);
        assert_eq!(
            midi_file.tracks[0].data[0].event,
            Event::MetaEvent(MetaEvent {
                content: MetaEventContent::TimeSignature {
                    numerator: 4,
                    denominator: 2,
                    clocks_per_metronome_click: 24,
                    thirty_seconds_per_quarter_note: 8,
                },
            })
        );
        assert_eq!(midi_file.tracks[0].data[1].delta_time.value, 0);
        assert_eq!(
            midi_file.tracks[0].data[1].event,
            Event::MetaEvent(MetaEvent {
                content: MetaEventContent::SetTempo { tempo: 500000 },
            })
        );
        assert_eq!(midi_file.tracks[0].data[2].delta_time.value, 384);
        assert_eq!(
            midi_file.tracks[0].data[2].event,
            Event::MetaEvent(MetaEvent {
                content: MetaEventContent::EndOfTrack,
            })
        );
        assert_eq!(midi_file.tracks[1].chunk_type, "MTrk");
        assert_eq!(midi_file.tracks[1].length, 16);
        assert_eq!(midi_file.tracks[1].data.len(), 4);
        assert_eq!(midi_file.tracks[1].data[0].delta_time.value, 0);
        assert_eq!(
            midi_file.tracks[1].data[0].event,
            Event::MidiEvent(MidiMessage::ChannelMessage(
                ChannelMessage::ChannelVoiceMessage(ChannelVoiceMessage::ProgramChange {
                    channel: 0,
                    program: 5
                })
            ))
        );
        assert_eq!(midi_file.tracks[1].data[1].delta_time.value, 192);
        assert_eq!(
            midi_file.tracks[1].data[1].event,
            Event::MidiEvent(MidiMessage::ChannelMessage(
                ChannelMessage::ChannelVoiceMessage(ChannelVoiceMessage::NoteOn {
                    channel: 0,
                    note: 76,
                    velocity: 32
                })
            ))
        );
        assert_eq!(midi_file.tracks[1].data[2].delta_time.value, 192);
        assert_eq!(
            midi_file.tracks[1].data[2].event,
            Event::MidiEvent(MidiMessage::ChannelMessage(
                ChannelMessage::ChannelVoiceMessage(ChannelVoiceMessage::NoteOn {
                    channel: 0,
                    note: 76,
                    velocity: 0
                })
            ))
        );
        assert_eq!(midi_file.tracks[1].data[3].delta_time.value, 0);
        assert_eq!(
            midi_file.tracks[1].data[3].event,
            Event::MetaEvent(MetaEvent {
                content: MetaEventContent::EndOfTrack,
            })
        );
        assert_eq!(midi_file.tracks[2].chunk_type, "MTrk");
        assert_eq!(midi_file.tracks[2].length, 15);
        assert_eq!(midi_file.tracks[2].data.len(), 4);
        assert_eq!(midi_file.tracks[2].data[0].delta_time.value, 0);
        assert_eq!(
            midi_file.tracks[2].data[0].event,
            Event::MidiEvent(MidiMessage::ChannelMessage(
                ChannelMessage::ChannelVoiceMessage(ChannelVoiceMessage::ProgramChange {
                    channel: 1,
                    program: 46
                })
            ))
        );
        assert_eq!(midi_file.tracks[2].data[1].delta_time.value, 96);
        assert_eq!(
            midi_file.tracks[2].data[1].event,
            Event::MidiEvent(MidiMessage::ChannelMessage(
                ChannelMessage::ChannelVoiceMessage(ChannelVoiceMessage::NoteOn {
                    channel: 1,
                    note: 67,
                    velocity: 64
                })
            ))
        );
        assert_eq!(midi_file.tracks[2].data[2].delta_time.value, 288);
        assert_eq!(
            midi_file.tracks[2].data[2].event,
            Event::MidiEvent(MidiMessage::ChannelMessage(
                ChannelMessage::ChannelVoiceMessage(ChannelVoiceMessage::NoteOn {
                    channel: 1,
                    note: 67,
                    velocity: 0
                })
            ))
        );
        assert_eq!(midi_file.tracks[2].data[3].delta_time.value, 0);
        assert_eq!(
            midi_file.tracks[2].data[3].event,
            Event::MetaEvent(MetaEvent {
                content: MetaEventContent::EndOfTrack,
            })
        );
        assert_eq!(midi_file.tracks[3].chunk_type, "MTrk");
        assert_eq!(midi_file.tracks[3].length, 21);
        assert_eq!(midi_file.tracks[3].data.len(), 6);
        assert_eq!(midi_file.tracks[3].data[0].delta_time.value, 0);
        assert_eq!(
            midi_file.tracks[3].data[0].event,
            Event::MidiEvent(MidiMessage::ChannelMessage(
                ChannelMessage::ChannelVoiceMessage(ChannelVoiceMessage::ProgramChange {
                    channel: 2,
                    program: 70
                })
            ))
        );
        assert_eq!(midi_file.tracks[3].data[1].delta_time.value, 0);
        assert_eq!(
            midi_file.tracks[3].data[1].event,
            Event::MidiEvent(MidiMessage::ChannelMessage(
                ChannelMessage::ChannelVoiceMessage(ChannelVoiceMessage::NoteOn {
                    channel: 2,
                    note: 48,
                    velocity: 96
                })
            ))
        );
        assert_eq!(midi_file.tracks[3].data[2].delta_time.value, 0);
        assert_eq!(
            midi_file.tracks[3].data[2].event,
            Event::MidiEvent(MidiMessage::ChannelMessage(
                ChannelMessage::ChannelVoiceMessage(ChannelVoiceMessage::NoteOn {
                    channel: 2,
                    note: 60,
                    velocity: 96
                })
            ))
        );
        assert_eq!(midi_file.tracks[3].data[3].delta_time.value, 384);
        assert_eq!(
            midi_file.tracks[3].data[3].event,
            Event::MidiEvent(MidiMessage::ChannelMessage(
                ChannelMessage::ChannelVoiceMessage(ChannelVoiceMessage::NoteOn {
                    channel: 2,
                    note: 48,
                    velocity: 0
                })
            ))
        );
        assert_eq!(midi_file.tracks[3].data[4].delta_time.value, 0);
        assert_eq!(
            midi_file.tracks[3].data[4].event,
            Event::MidiEvent(MidiMessage::ChannelMessage(
                ChannelMessage::ChannelVoiceMessage(ChannelVoiceMessage::NoteOn {
                    channel: 2,
                    note: 60,
                    velocity: 0
                })
            ))
        );
        assert_eq!(midi_file.tracks[3].data[5].delta_time.value, 0);
        assert_eq!(
            midi_file.tracks[3].data[5].event,
            Event::MetaEvent(MetaEvent {
                content: MetaEventContent::EndOfTrack,
            })
        );
    }
}
