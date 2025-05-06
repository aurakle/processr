use nom::{character::anychar, combinator::{peek, recognize, rest, verify}, error::Error, multi::many_till, IResult, Parser};

/// Return the remaining input.
///
/// This parser is similar to [`nom::combinator::rest`], but returns `Err(Err::Error((_, ErrorKind::Verify)))` if the input is empty.
pub fn rest1(s: &str) -> IResult<&str, &str> {
    verify(rest, |x: &str| !x.is_empty())(s)
}

/// Returns the *shortest* input slice until it matches a parser.
///
/// Returns `Err(Err::Error((_, ErrorKind::Eof)))` if the input doesn't match the parser.
pub fn take_before0<'a, FOutput, F>(f: F) -> impl Parser<&'a str, Output = FOutput, Error = Error<&'a str>>
where
    F: Parser<&'a str, Output = FOutput, Error = Error<&'a str>>,
{
    recognize(many_till(anychar, peek(f)))
}

/// Returns the *shortest* input slice until it matches a parser.
///
/// This parser is similar to [`take_before0`], but must return at least one character.
///
/// Returns `Err(Err::Error((_, ErrorKind::Eof)))` if the input doesn't match the parser.
///
/// Returns `Err(Err::Error((_, ErrorKind::Verify)))` if the input itself matches the parser
/// (i.e. this parser cannot return any characters).
pub fn take_before1<'a, FOutput, F>(f: F) -> impl Parser<&'a str, Output = FOutput, Error = Error<&'a str>>
where
    F: Parser<&'a str, Output = FOutput, Error = Error<&'a str>>,
{
    verify(take_before0(f), |x: &str| !x.is_empty())
}
