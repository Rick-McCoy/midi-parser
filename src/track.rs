use crate::event::MTrkEvent;
use nom::{
    bytes::complete::{tag, take},
    number::complete::be_u32,
    IResult,
};

#[derive(PartialEq, Debug)]
pub struct TrackChunk {
    pub chunk_type: String,
    pub length: u32,
    pub data: Vec<MTrkEvent>,
}

impl TrackChunk {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, chunk_type) = tag("MTrk")(input)?;
        let chunk_type = String::from_utf8(chunk_type.to_vec()).expect("Invalid chunk type");
        let (input, length) = be_u32(input)?;
        let (input, mut bytes) = take(length as usize)(input)?;
        let mut data: Vec<MTrkEvent> = Vec::new();
        while bytes.len() > 0 {
            let (remaining, event) = MTrkEvent::parse(
                bytes,
                match data.last() {
                    Some(event) => event.get_status(),
                    None => 0xff,
                },
            )?;
            data.push(event);
            bytes = remaining;
        }
        Ok((
            input,
            Self {
                chunk_type,
                length,
                data,
            },
        ))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let chunk_bytes = self.chunk_type.as_bytes().to_vec();
        let length_bytes = self.length.to_be_bytes().to_vec();
        let data_bytes = self
            .data
            .iter()
            .flat_map(|event| event.to_bytes())
            .collect::<Vec<u8>>();
        assert_eq!(self.length as usize, data_bytes.len());
        [chunk_bytes, length_bytes, data_bytes].concat()
    }
}
