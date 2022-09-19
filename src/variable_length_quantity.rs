use nom::{
    bytes::complete::{take, take_while},
    sequence::tuple,
    IResult,
};
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct VariableLengthQuantity {
    pub value: u32,
}

impl VariableLengthQuantity {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (byte_prefix, last_byte)) =
            tuple((take_while(|byte| byte & 0x80 != 0), take(1usize)))(input)?;

        let value = byte_prefix
            .iter()
            .chain(last_byte.iter())
            .fold(0, |acc, byte| (acc << 7) | (byte & 0x7f) as u32);

        Ok((input, Self { value }))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut value = self.value;
        loop {
            let byte = (value & 0x7f) as u8;
            value >>= 7;
            if bytes.is_empty() {
                bytes.push(byte);
            } else {
                bytes.push(byte | 0x80);
            }
            if value == 0 {
                break;
            }
        }
        bytes.reverse();
        bytes
    }
}

#[cfg(test)]
mod tests {
    pub use super::VariableLengthQuantity;
    use nom::Finish;

    #[test]
    fn test_parse_variable_length_quantity() {
        let input = [0x00];
        let expected = VariableLengthQuantity { value: 0x00 };
        let (_, actual) = VariableLengthQuantity::parse(&input).finish().unwrap();
        assert_eq!(expected, actual);

        let input = [0x40];
        let expected = VariableLengthQuantity { value: 0x40 };
        let actual = VariableLengthQuantity::parse(&input).finish().unwrap().1;
        assert_eq!(expected, actual);

        let input = [0x7f];
        let expected = VariableLengthQuantity { value: 0x7f };
        let actual = VariableLengthQuantity::parse(&input).finish().unwrap().1;
        assert_eq!(expected, actual);

        let input = [0x81, 0x00];
        let expected = VariableLengthQuantity { value: 0x80 };
        let actual = VariableLengthQuantity::parse(&input).finish().unwrap().1;
        assert_eq!(expected, actual);

        let input = [0xc0, 0x00];
        let expected = VariableLengthQuantity { value: 0x2000 };
        let actual = VariableLengthQuantity::parse(&input).finish().unwrap().1;
        assert_eq!(expected, actual);

        let input = [0xff, 0x7f];
        let expected = VariableLengthQuantity { value: 0x3fff };
        let actual = VariableLengthQuantity::parse(&input).finish().unwrap().1;
        assert_eq!(expected, actual);

        let input = [0x81, 0x80, 0x00];
        let expected = VariableLengthQuantity { value: 0x4000 };
        let actual = VariableLengthQuantity::parse(&input).finish().unwrap().1;
        assert_eq!(expected, actual);

        let input = [0xc0, 0x80, 0x00];
        let expected = VariableLengthQuantity { value: 0x100000 };
        let actual = VariableLengthQuantity::parse(&input).finish().unwrap().1;
        assert_eq!(expected, actual);

        let input = [0xff, 0xff, 0x7f];
        let expected = VariableLengthQuantity { value: 0x1fffff };
        let actual = VariableLengthQuantity::parse(&input).finish().unwrap().1;
        assert_eq!(expected, actual);

        let input = [0x81, 0x80, 0x80, 0x00];
        let expected = VariableLengthQuantity { value: 0x200000 };
        let actual = VariableLengthQuantity::parse(&input).finish().unwrap().1;
        assert_eq!(expected, actual);

        let input = [0xc0, 0x80, 0x80, 0x00];
        let expected = VariableLengthQuantity { value: 0x8000000 };
        let actual = VariableLengthQuantity::parse(&input).finish().unwrap().1;
        assert_eq!(expected, actual);

        let input = [0xff, 0xff, 0xff, 0x7f];
        let expected = VariableLengthQuantity { value: 0xfffffff };
        let actual = VariableLengthQuantity::parse(&input).finish().unwrap().1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_to_bytes() {
        let input = VariableLengthQuantity { value: 0x00 };
        let expected = vec![0x00u8];
        let actual = input.to_bytes();
        assert_eq!(expected, actual);

        let input = VariableLengthQuantity { value: 0x40 };
        let expected = vec![0x40u8];
        let actual = input.to_bytes();
        assert_eq!(expected, actual);

        let input = VariableLengthQuantity { value: 0x7f };
        let expected = vec![0x7fu8];
        let actual = input.to_bytes();
        assert_eq!(expected, actual);

        let input = VariableLengthQuantity { value: 0x80 };
        let expected = vec![0x81u8, 0x00u8];
        let actual = input.to_bytes();
        assert_eq!(expected, actual);

        let input = VariableLengthQuantity { value: 0x2000 };
        let expected = vec![0xc0u8, 0x00u8];
        let actual = input.to_bytes();
        assert_eq!(expected, actual);

        let input = VariableLengthQuantity { value: 0x3fff };
        let expected = vec![0xffu8, 0x7fu8];
        let actual = input.to_bytes();
        assert_eq!(expected, actual);

        let input = VariableLengthQuantity { value: 0x4000 };
        let expected = vec![0x81u8, 0x80u8, 0x00u8];
        let actual = input.to_bytes();
        assert_eq!(expected, actual);

        let input = VariableLengthQuantity { value: 0x100000 };
        let expected = vec![0xc0u8, 0x80u8, 0x00u8];
        let actual = input.to_bytes();
        assert_eq!(expected, actual);

        let input = VariableLengthQuantity { value: 0x1fffff };
        let expected = vec![0xffu8, 0xffu8, 0x7fu8];
        let actual = input.to_bytes();
        assert_eq!(expected, actual);

        let input = VariableLengthQuantity { value: 0x200000 };
        let expected = vec![0x81u8, 0x80u8, 0x80u8, 0x00u8];
        let actual = input.to_bytes();
        assert_eq!(expected, actual);

        let input = VariableLengthQuantity { value: 0x8000000 };
        let expected = vec![0xc0u8, 0x80u8, 0x80u8, 0x00u8];
        let actual = input.to_bytes();
        assert_eq!(expected, actual);

        let input = VariableLengthQuantity { value: 0xfffffff };
        let expected = vec![0xffu8, 0xffu8, 0xffu8, 0x7fu8];
        let actual = input.to_bytes();
        assert_eq!(expected, actual);
    }
}
