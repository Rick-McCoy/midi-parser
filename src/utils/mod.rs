use nom::{combinator::verify, number::complete::be_u8, IResult};

pub fn be_u7(input: &[u8]) -> IResult<&[u8], u8> {
    Ok(verify(be_u8, |value| value >> 7 == 0)(input)?)
}
