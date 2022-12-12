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

        assert!(last_byte.len() == 1);
        let last_byte = last_byte[0];
        assert!(last_byte & 0x80 == 0);

        assert!(byte_prefix.len() < 4);

        let value = byte_prefix
            .iter()
            .chain(std::iter::once(&last_byte))
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

    #[test]
    fn test_parse_variable_length_quantity() {
        let input_answer_pairs_1 = [([0x00], 0x00), ([0x40], 0x40), ([0x7f], 0x7f)];
        let input_answer_pairs_2 = [
            ([0x81, 0x00], 0x80),
            ([0xc0, 0x00], 0x2000),
            ([0xff, 0x7f], 0x3fff),
        ];
        let input_answer_pairs_3 = [
            ([0x81, 0x80, 0x00], 0x4000),
            ([0xc0, 0x80, 0x00], 0x100000),
            ([0xff, 0xff, 0x7f], 0x1fffff),
        ];
        let input_answer_pairs_4 = [
            ([0x81, 0x80, 0x80, 0x00], 0x200000),
            ([0xc0, 0x80, 0x80, 0x00], 0x8000000),
            ([0xff, 0xff, 0xff, 0x7f], 0xfffffff),
        ];
        for (input, answer) in input_answer_pairs_1.iter() {
            let expected = VariableLengthQuantity { value: *answer };
            let actual = match VariableLengthQuantity::parse(input) {
                Ok((_, actual)) => actual,
                Err(e) => panic!("{:?}", e),
            };
            assert_eq!(expected, actual);
        }
        for (input, answer) in input_answer_pairs_2.iter() {
            let expected = VariableLengthQuantity { value: *answer };
            let actual = match VariableLengthQuantity::parse(input) {
                Ok((_, actual)) => actual,
                Err(e) => panic!("{:?}", e),
            };
            assert_eq!(expected, actual);
        }
        for (input, answer) in input_answer_pairs_3.iter() {
            let expected = VariableLengthQuantity { value: *answer };
            let actual = match VariableLengthQuantity::parse(input) {
                Ok((_, actual)) => actual,
                Err(e) => panic!("{:?}", e),
            };
            assert_eq!(expected, actual);
        }
        for (input, answer) in input_answer_pairs_4.iter() {
            let expected = VariableLengthQuantity { value: *answer };
            let actual = match VariableLengthQuantity::parse(input) {
                Ok((_, actual)) => actual,
                Err(e) => panic!("{:?}", e),
            };
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn test_to_bytes() {
        let input_answer_pairs_1 = [(0x00, [0x00]), (0x40, [0x40]), (0x7f, [0x7f])];
        let input_answer_pairs_2 = [
            (0x80, [0x81, 0x00]),
            (0x2000, [0xc0, 0x00]),
            (0x3fff, [0xff, 0x7f]),
        ];
        let input_answer_pairs_3 = [
            (0x4000, [0x81, 0x80, 0x00]),
            (0x100000, [0xc0, 0x80, 0x00]),
            (0x1fffff, [0xff, 0xff, 0x7f]),
        ];
        let input_answer_pairs_4 = [
            (0x200000, [0x81, 0x80, 0x80, 0x00]),
            (0x8000000, [0xc0, 0x80, 0x80, 0x00]),
            (0xfffffff, [0xff, 0xff, 0xff, 0x7f]),
        ];
        for (input, answer) in input_answer_pairs_1.iter() {
            let expected = answer.to_vec();
            let actual = VariableLengthQuantity { value: *input }.to_bytes();
            assert_eq!(expected, actual);
        }
        for (input, answer) in input_answer_pairs_2.iter() {
            let expected = answer.to_vec();
            let actual = VariableLengthQuantity { value: *input }.to_bytes();
            assert_eq!(expected, actual);
        }
        for (input, answer) in input_answer_pairs_3.iter() {
            let expected = answer.to_vec();
            let actual = VariableLengthQuantity { value: *input }.to_bytes();
            assert_eq!(expected, actual);
        }
        for (input, answer) in input_answer_pairs_4.iter() {
            let expected = answer.to_vec();
            let actual = VariableLengthQuantity { value: *input }.to_bytes();
            assert_eq!(expected, actual);
        }
    }
}
