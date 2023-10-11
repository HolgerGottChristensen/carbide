use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take};
use nom::combinator::{map, not};
use nom::IResult;
use nom::multi::{many0, many1};
use nom::sequence::{delimited, preceded, tuple};

use crate::environment::Environment;
use crate::text::{FontStyle, FontWeight, TextSpanGenerator};
use crate::text::text_span::TextSpan;
use crate::text::text_style::TextStyle;
use crate::text::types::text_decoration::TextDecoration;

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
    //LineSeparator,
    Newline,
    Paragraph(String),
}

pub fn parse_polar_bear_markup(input: &str) -> IResult<&str, Vec<PolarItem>> {
    let (left, parsed) = many0(alt((
        parse_header_1,
        parse_header_2,
        parse_header_3,
        parse_header_4,
        parse_header_5,
        parse_header_6,
        parse_underline,
        parse_strike_through,
        parse_italic,
        parse_bold,
        parse_paragraph,
        parse_newline,
    )))(input)?;

    Ok((left, parsed))
}

// #[test]
// fn parse_polar_bear_markup_test() {
//     assert_eq!(
//         parse_polar_bear_markup("Hejsa1"),
//         Ok(("", vec![PolarItem::Paragraph("Hejsa1".to_string())]))
//     );
//     assert_eq!(
//         parse_polar_bear_markup("/Hejsa2/"),
//         Ok(("", vec![PolarItem::Italic("Hejsa2".to_string())]))
//     );
//     assert_eq!(
//         parse_polar_bear_markup("/Hejsa3/ verden!"),
//         Ok((
//             "",
//             vec![
//                 PolarItem::Italic("Hejsa3".to_string()),
//                 PolarItem::Paragraph(" verden!".to_string()),
//             ]
//         ))
//     );
//     assert_eq!(
//         parse_polar_bear_markup("Hejsa4 /verden!/"),
//         Ok((
//             "",
//             vec![
//                 PolarItem::Paragraph("Hejsa4 ".to_string()),
//                 PolarItem::Italic("verden!".to_string()),
//             ]
//         ))
//     );
//     assert_eq!(
//         parse_polar_bear_markup("/Hejsa5 / verden!"),
//         Ok((
//             "",
//             vec![PolarItem::Paragraph("/Hejsa5 / verden!".to_string())]
//         ))
//     );
// }

fn parse_header_1(input: &str) -> IResult<&str, PolarItem> {
    let (left, (_, _, parsed, _)): (&str, (_, _, &str, _)) =
        tuple((tag("#"), tag(" "), is_not("\n"), tag("\n")))(input)?;

    Ok((left, PolarItem::Header1(parsed.to_string())))
}

fn parse_header_2(input: &str) -> IResult<&str, PolarItem> {
    let (left, (_, _, parsed, _)): (&str, (_, _, &str, _)) =
        tuple((tag("##"), tag(" "), is_not("\n"), tag("\n")))(input)?;

    Ok((left, PolarItem::Header2(parsed.to_string())))
}

fn parse_header_3(input: &str) -> IResult<&str, PolarItem> {
    let (left, (_, _, parsed, _)): (&str, (_, _, &str, _)) =
        tuple((tag("###"), tag(" "), is_not("\n"), tag("\n")))(input)?;

    Ok((left, PolarItem::Header3(parsed.to_string())))
}

fn parse_header_4(input: &str) -> IResult<&str, PolarItem> {
    let (left, (_, _, parsed, _)): (&str, (_, _, &str, _)) =
        tuple((tag("####"), tag(" "), is_not("\n"), tag("\n")))(input)?;

    Ok((left, PolarItem::Header4(parsed.to_string())))
}

fn parse_header_5(input: &str) -> IResult<&str, PolarItem> {
    let (left, (_, _, parsed, _)): (&str, (_, _, &str, _)) =
        tuple((tag("#####"), tag(" "), is_not("\n"), tag("\n")))(input)?;

    Ok((left, PolarItem::Header5(parsed.to_string())))
}

fn parse_header_6(input: &str) -> IResult<&str, PolarItem> {
    let (left, (_, _, parsed, _)): (&str, (_, _, &str, _)) =
        tuple((tag("######"), tag(" "), is_not("\n"), tag("\n")))(input)?;

    Ok((left, PolarItem::Header6(parsed.to_string())))
}

fn parse_newline(input: &str) -> IResult<&str, PolarItem> {
    let (left, _parsed): (&str, &str) = tag("\n")(input)?;

    Ok((left, PolarItem::Newline))
}

fn parse_underline(input: &str) -> IResult<&str, PolarItem> {
    let (left, parsed): (&str, String) = delimited(tag("_"), parse_text, tag("_"))(input)?;
    let italic = PolarItem::Underline(parsed);
    Ok((left, italic))
}

fn parse_strike_through(input: &str) -> IResult<&str, PolarItem> {
    let (left, parsed): (&str, String) = delimited(tag("-"), parse_text, tag("-"))(input)?;
    let italic = PolarItem::Strike(parsed);
    Ok((left, italic))
}

fn parse_italic(input: &str) -> IResult<&str, PolarItem> {
    let (left, parsed): (&str, String) = delimited(tag("/"), parse_text, tag("/"))(input)?;
    let italic = PolarItem::Italic(parsed);
    Ok((left, italic))
}

fn parse_bold(input: &str) -> IResult<&str, PolarItem> {
    let (left, parsed): (&str, String) = delimited(tag("*"), parse_text, tag("*"))(input)?;
    let italic = PolarItem::Bold(parsed);
    Ok((left, italic))
}

fn parse_paragraph(input: &str) -> IResult<&str, PolarItem> {
    let (left, parsed): (&str, String) = parse_text(input)?;

    let paragraph = PolarItem::Paragraph(parsed);
    Ok((left, paragraph))
}

fn parse_text(input: &str) -> IResult<&str, String> {
    let (left, parsed): (&str, String) = map(
        many1(preceded(
            not(alt((tag("/"), tag("*"), tag("-"), tag("_"), tag("\n")))),
            take(1u8),
        )),
        |vec| vec.join(""),
    )(input)?;

    Ok((left, parsed))
}

#[derive(Debug, Clone)]
pub struct PolarBearMarkup;

impl PolarBearMarkup {
    pub fn new() -> PolarBearMarkup {
        PolarBearMarkup {}
    }
}

impl TextSpanGenerator for PolarBearMarkup {
    // https://bear.app/faq/Markup%20:%20Markdown/Polar%20Bear%20markup%20language/
    fn generate(&self, string: &str, style: &TextStyle, env: &mut Environment) -> Vec<TextSpan> {
        let default_font_family_name = &style.font_family;
        let scale_factor = env.scale_factor();
        let polars = parse_polar_bear_markup(string).unwrap().1;

        let mut spans = vec![];

        for polar in polars {
            match polar {
                PolarItem::Header1(text) => {
                    let style = TextStyle {
                        font_family: default_font_family_name.clone(),
                        font_size: 30,
                        font_style: FontStyle::Normal,
                        font_weight: FontWeight::Bold,
                        text_decoration: TextDecoration::None,
                        color: None,
                    };
                    let font = style.get_font(env);

                    let (widths, glyphs) =
                        font.glyphs_for(&text, style.font_size, scale_factor, env);
                    let ascending_pixels = font.ascend(style.font_size, scale_factor);
                    let line_height = font.descend(style.font_size, scale_factor);
                    let line_gap = font.line_gap(style.font_size, scale_factor);

                    let span = TextSpan::Text {
                        style: Some(style.clone()),
                        text: text.to_string(),
                        glyphs,
                        widths,
                        ascend: ascending_pixels,
                        descend: line_height,
                        line_gap,
                    };

                    spans.push(span);
                    spans.push(TextSpan::NewLine)
                }
                PolarItem::Header2(text) => {
                    let style = TextStyle {
                        font_family: default_font_family_name.clone(),
                        font_size: 20,
                        font_style: FontStyle::Normal,
                        font_weight: FontWeight::Normal,
                        text_decoration: TextDecoration::None,
                        color: None,
                    };
                    let font = style.get_font(env);

                    let (widths, glyphs) =
                        font.glyphs_for(&text, style.font_size, scale_factor, env);
                    let ascending_pixels = font.ascend(style.font_size, scale_factor);
                    let line_height = font.descend(style.font_size, scale_factor);

                    let line_gap = font.line_gap(style.font_size, scale_factor);

                    let span = TextSpan::Text {
                        style: Some(style.clone()),
                        text: text.to_string(),
                        glyphs,
                        widths,
                        ascend: ascending_pixels,
                        descend: line_height,
                        line_gap,
                    };

                    spans.push(span);
                    spans.push(TextSpan::NewLine)
                }
                PolarItem::Italic(text) => {
                    let style = TextStyle {
                        font_family: default_font_family_name.clone(),
                        font_size: 14,
                        font_style: FontStyle::Italic,
                        font_weight: FontWeight::Normal,
                        text_decoration: TextDecoration::None,
                        color: None,
                    };
                    let font = style.get_font(env);

                    let (widths, glyphs) =
                        font.glyphs_for(&text, style.font_size, scale_factor, env);
                    let ascending_pixels = font.ascend(style.font_size, scale_factor);
                    let line_height = font.descend(style.font_size, scale_factor);

                    let line_gap = font.line_gap(style.font_size, scale_factor);

                    let span = TextSpan::Text {
                        style: Some(style.clone()),
                        text: text.to_string(),
                        glyphs,
                        widths,
                        ascend: ascending_pixels,
                        descend: line_height,
                        line_gap,
                    };

                    spans.push(span);
                }
                PolarItem::Bold(text) => {
                    let style = TextStyle {
                        font_family: default_font_family_name.clone(),
                        font_size: 14,
                        font_style: FontStyle::Normal,
                        font_weight: FontWeight::Bold,
                        text_decoration: TextDecoration::None,
                        color: None,
                    };
                    let font = style.get_font(env);

                    let (widths, glyphs) =
                        font.glyphs_for(&text, style.font_size, scale_factor, env);
                    let ascending_pixels = font.ascend(style.font_size, scale_factor);
                    let line_height = font.descend(style.font_size, scale_factor);
                    let line_gap = font.line_gap(style.font_size, scale_factor);

                    let span = TextSpan::Text {
                        style: Some(style.clone()),
                        text: text.to_string(),
                        glyphs,
                        widths,
                        ascend: ascending_pixels,
                        descend: line_height,
                        line_gap,
                    };

                    spans.push(span);
                }
                PolarItem::Paragraph(text) => {
                    let style = TextStyle {
                        font_family: default_font_family_name.clone(),
                        font_size: 14,
                        font_style: FontStyle::Normal,
                        font_weight: FontWeight::Normal,
                        text_decoration: TextDecoration::None,
                        color: None,
                    };
                    let font = style.get_font(env);

                    let (widths, glyphs) =
                        font.glyphs_for(&text, style.font_size, scale_factor, env);
                    let ascending_pixels = font.ascend(style.font_size, scale_factor);
                    let line_height = font.descend(style.font_size, scale_factor);
                    let line_gap = font.line_gap(style.font_size, scale_factor);

                    let span = TextSpan::Text {
                        style: Some(style.clone()),
                        text: text.to_string(),
                        glyphs,
                        widths,
                        ascend: ascending_pixels,
                        descend: line_height,
                        line_gap,
                    };

                    spans.push(span);
                }
                PolarItem::Underline(text) => {
                    let style = TextStyle {
                        font_family: default_font_family_name.clone(),
                        font_size: 14,
                        font_style: FontStyle::Normal,
                        font_weight: FontWeight::Normal,
                        text_decoration: TextDecoration::Underline(vec![]),
                        color: None,
                    };
                    let font = style.get_font(env);

                    let (widths, glyphs) =
                        font.glyphs_for(&text, style.font_size, scale_factor, env);
                    let ascending_pixels = font.ascend(style.font_size, scale_factor);
                    let line_height = font.descend(style.font_size, scale_factor);
                    let line_gap = font.line_gap(style.font_size, scale_factor);

                    let span = TextSpan::Text {
                        style: Some(style.clone()),
                        text: text.to_string(),
                        glyphs,
                        widths,
                        ascend: ascending_pixels,
                        descend: line_height,
                        line_gap,
                    };

                    spans.push(span);
                }
                PolarItem::Strike(text) => {
                    let style = TextStyle {
                        font_family: default_font_family_name.clone(),
                        font_size: 14,
                        font_style: FontStyle::Normal,
                        font_weight: FontWeight::Normal,
                        text_decoration: TextDecoration::StrikeThrough(vec![]),
                        color: None,
                    };
                    let font = style.get_font(env);

                    let (widths, glyphs) =
                        font.glyphs_for(&text, style.font_size, scale_factor, env);
                    let ascending_pixels = font.ascend(style.font_size, scale_factor);
                    let line_height = font.descend(style.font_size, scale_factor);
                    let line_gap = font.line_gap(style.font_size, scale_factor);

                    let span = TextSpan::Text {
                        style: Some(style.clone()),
                        text: text.to_string(),
                        glyphs,
                        widths,
                        ascend: ascending_pixels,
                        descend: line_height,
                        line_gap,
                    };

                    spans.push(span);
                }
                PolarItem::Newline => spans.push(TextSpan::NewLine),
                _ => (),
            }
        }

        spans
    }

    fn store_color(&self) -> bool {
        true
    }
}

impl Into<Box<dyn TextSpanGenerator>> for PolarBearMarkup {
    fn into(self) -> Box<dyn TextSpanGenerator> {
        Box::new(self)
    }
}
