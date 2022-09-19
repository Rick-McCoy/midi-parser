use nom::{
    bytes::complete::{tag, take},
    combinator::opt,
    number::complete::be_u8,
    IResult,
};

use crate::variable_length_quantity::VariableLengthQuantity;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SysExEvent {
    prefix: u8,
    data: Vec<u8>,
    suffix: Option<u8>,
}

impl SysExEvent {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, prefix) = be_u8(input)?;
        let (input, len) = VariableLengthQuantity::parse(input)?;
        let (input, data) = take(len.value as usize)(input)?;
        let (input, suffix) = opt(tag(&[0xF7]))(input)?;
        Ok((
            input,
            Self {
                prefix,
                data: data.to_vec(),
                suffix: suffix.map(|x| x[0]),
            },
        ))
    }

    pub fn get_status(&self) -> u8 {
        self.prefix
    }
}
