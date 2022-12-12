use nom::{
    bytes::complete::{tag, take},
    IResult,
};

use crate::variable_length_quantity::VariableLengthQuantity;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SysExEvent {
    prefix: u8,
    data: Vec<u8>,
}

impl SysExEvent {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, prefix) = tag(&[0xf0, 0xf7])(input)?;
        let (input, len) = VariableLengthQuantity::parse(input)?;
        let (input, data) = take(len.value as usize)(input)?;
        let (last_byte, penultimate_data) = match data.split_last() {
            Some((last_byte, penultimate_data)) => (last_byte, penultimate_data),
            None => {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Verify,
                )))
            }
        };
        penultimate_data
            .iter()
            .for_each(|byte| assert_eq!(byte >> 7, 0, "Invalid data: {:x?}", data));
        assert!(
            *last_byte >> 7 == 0 || *last_byte == 0xf7,
            "Data: {:x?}",
            data
        );
        Ok((
            input,
            Self {
                prefix: prefix[0],
                data: data.to_vec(),
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
        bytes
    }

    pub fn get_status(&self) -> u8 {
        self.prefix
    }
}
