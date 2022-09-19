use nom::{
    bytes::complete::{tag, take},
    number::complete::{be_u16, be_u32},
    IResult,
};

#[derive(PartialEq, Debug)]
pub struct HeaderData {
    pub format: u16,
    pub ntrks: u16,
    pub division: u16,
}

impl HeaderData {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, format) = be_u16(input)?;
        assert!(format == 0 || format == 1 || format == 2);
        let (input, ntrks) = be_u16(input)?;
        let (input, division) = be_u16(input)?;
        Ok((
            input,
            Self {
                format,
                ntrks,
                division,
            },
        ))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        [
            self.format.to_be_bytes(),
            self.ntrks.to_be_bytes(),
            self.division.to_be_bytes(),
        ]
        .concat()
    }
}

#[derive(PartialEq, Debug)]
pub struct HeaderChunk {
    pub chunk_type: String,
    pub length: u32,
    pub data: HeaderData,
}

impl HeaderChunk {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, chunk_type) = tag("MThd")(input)?;
        let (input, length) = be_u32(input)?;
        let (input, header_slice) = take(length as usize)(input)?;
        let (_, data) = HeaderData::parse(header_slice)?;
        Ok((
            input,
            Self {
                chunk_type: String::from_utf8(chunk_type.to_vec()).expect("Invalid chunk type"),
                length,
                data,
            },
        ))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        [
            self.chunk_type.as_bytes(),
            &self.length.to_be_bytes(),
            &self.data.to_bytes(),
        ]
        .concat()
    }
}
