use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take, take_until};
use nom::character::complete::{alphanumeric1, none_of, space1};
use nom::combinator::{eof, map, not, opt, peek};
use nom::IResult;
use nom::multi::{many0, many1};
use nom::sequence::{delimited, preceded, tuple};

#[derive(PartialEq, Debug, Clone)]
pub enum PolarItem {
    Header1(String),
    Header2(String),
    Header3(String),
    Header4(String),
    Header5(String),
    Header6(String),
    Italic(String),
    Bold(String),
    Underline(String),
    Strike(String),
    LineSeparator,
    Newline,
    Paragraph(String),
}

pub fn parse_polar_bear_markup(input: &str) -> IResult<&str, Vec<PolarItem>> {
    let (left, parsed) = many0(alt((
        parse_header_1,
        parse_header_2,
        parse_underline,
        parse_strike_through,
        parse_italic,
        parse_bold,
        parse_paragraph,
        parse_newline,
    )))(input)?;

    Ok((left, parsed))
}

#[test]
fn parse_polar_bear_markup_test() {
    assert_eq!(parse_polar_bear_markup("Hejsa1"), Ok(("", vec![PolarItem::Paragraph("Hejsa1".to_string())])));
    assert_eq!(parse_polar_bear_markup("/Hejsa2/"), Ok(("", vec![PolarItem::Italic("Hejsa2".to_string())])));
    assert_eq!(parse_polar_bear_markup("/Hejsa3/ verden!"), Ok(("", vec![PolarItem::Italic("Hejsa3".to_string()), PolarItem::Paragraph(" verden!".to_string())])));
    assert_eq!(parse_polar_bear_markup("Hejsa4 /verden!/"), Ok(("", vec![PolarItem::Paragraph("Hejsa4 ".to_string()), PolarItem::Italic("verden!".to_string())])));
    assert_eq!(parse_polar_bear_markup("/Hejsa5 / verden!"), Ok(("", vec![PolarItem::Paragraph("/Hejsa5 / verden!".to_string())])));
}

fn parse_header_1(input: &str) -> IResult<&str, PolarItem> {
    let (left, (_, _, parsed, _)): (&str, (_, _, &str, _)) = tuple((tag("#"), tag(" "), is_not(("\n")), tag("\n")))(input)?;

    Ok((left, PolarItem::Header1(parsed.to_string())))
}

fn parse_header_2(input: &str) -> IResult<&str, PolarItem> {
    let (left, (_, _, parsed, _)): (&str, (_, _, &str, _)) = tuple((tag("##"), tag(" "), is_not(("\n")), tag("\n")))(input)?;

    Ok((left, PolarItem::Header2(parsed.to_string())))
}

fn parse_header_3(input: &str) -> IResult<&str, PolarItem> {
    let (left, (_, _, parsed, _)): (&str, (_, _, &str, _)) = tuple((tag("###"), tag(" "), is_not(("\n")), tag("\n")))(input)?;

    Ok((left, PolarItem::Header3(parsed.to_string())))
}

fn parse_header_4(input: &str) -> IResult<&str, PolarItem> {
    let (left, (_, _, parsed, _)): (&str, (_, _, &str, _)) = tuple((tag("####"), tag(" "), is_not(("\n")), tag("\n")))(input)?;

    Ok((left, PolarItem::Header4(parsed.to_string())))
}

fn parse_header_5(input: &str) -> IResult<&str, PolarItem> {
    let (left, (_, _, parsed, _)): (&str, (_, _, &str, _)) = tuple((tag("#####"), tag(" "), is_not(("\n")), tag("\n")))(input)?;

    Ok((left, PolarItem::Header5(parsed.to_string())))
}

fn parse_header_6(input: &str) -> IResult<&str, PolarItem> {
    let (left, (_, _, parsed, _)): (&str, (_, _, &str, _)) = tuple((tag("######"), tag(" "), is_not(("\n")), tag("\n")))(input)?;

    Ok((left, PolarItem::Header6(parsed.to_string())))
}

fn parse_newline(input: &str) -> IResult<&str, PolarItem> {
    let (left, parsed): (&str, &str) = tag("\n")(input)?;

    Ok((left, PolarItem::Newline))
}

fn parse_underline(input: &str) -> IResult<&str, PolarItem> {
    let (left, parsed): (&str, String) =
        delimited(tag("_"), parse_text, tag("_"))(input)?;
    let italic = PolarItem::Underline(parsed);
    Ok((left, italic))
}

fn parse_strike_through(input: &str) -> IResult<&str, PolarItem> {
    let (left, parsed): (&str, String) =
        delimited(tag("-"), parse_text, tag("-"))(input)?;
    let italic = PolarItem::Strike(parsed);
    Ok((left, italic))
}

fn parse_italic(input: &str) -> IResult<&str, PolarItem> {
    let (left, parsed): (&str, String) =
        delimited(tag("/"), parse_text, tag("/"))(input)?;
    let italic = PolarItem::Italic(parsed);
    Ok((left, italic))
}

fn parse_bold(input: &str) -> IResult<&str, PolarItem> {
    let (left, parsed): (&str, String) =
        delimited(tag("*"), parse_text, tag("*"))(input)?;
    let italic = PolarItem::Bold(parsed);
    Ok((left, italic))
}

fn parse_paragraph(input: &str) -> IResult<&str, PolarItem> {
    let (left, parsed): (&str, String) = parse_text(input)?;

    let paragraph = PolarItem::Paragraph(parsed);
    Ok((left, paragraph))
}

fn parse_text(input: &str) -> IResult<&str, String> {
    let (left, parsed): (&str, String) =
        map(
            many1(preceded(
                not(alt((tag("/"), tag("*"), tag("-"), tag("_"), tag("\n")))),
                take(1u8),
            )),
            |vec| vec.join(""),
        )(input)?;

    Ok((left, parsed))
}