use nom::{
    bytes::complete::{tag, take},
    combinator::opt,
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
        let (input, prefix) = tag(&[0xf0])(input)?;
        let (input, len) = VariableLengthQuantity::parse(input)?;
        let (input, data) = take(len.value as usize)(input)?;
        data.iter()
            .for_each(|byte| assert_ne!(*byte >> 7, 1, "Length: {}, Data: {:?}", len.value, data));
        let (input, suffix) = opt(tag(&[0xf7]))(input)?;
        Ok((
            input,
            Self {
                prefix: prefix[0],
                data: data.to_vec(),
                suffix: suffix.map(|x| x[0]),
            },
        ))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.prefix);
        let len = VariableLengthQuantity {
            value: self.data.len() as u32,
        };
        bytes.extend(len.to_bytes());
        bytes.extend(self.data.iter());
        if let Some(suffix) = self.suffix {
            bytes.push(suffix);
        }
        bytes
    }

    pub fn get_status(&self) -> u8 {
        self.prefix
    }
}
