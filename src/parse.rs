use std::{cell::OnceCell, iter::repeat_n, num::NonZeroU32, ops::Range, str::FromStr};

use crate::{
    color_table::parse_color,
    misc::{Style, Weight},
    parse_util::{
        ConditionOutput, Flip, ParseBuilder, ParseConditionFn, ParseError, ParseErrorType,
        ParseStyleFn, ParseValueFn,
    },
    SegmentSize, SegmentStyle, Text3d, Text3dSegment,
};

fn trim_mut(s: &mut String) {
    let trimmed = s.trim();
    let start = trimmed.as_ptr() as usize - s.as_ptr() as usize;
    let end = start + trimmed.len();

    s.truncate(end);
    s.drain(..start);
}

fn sub_span(origin: usize, sub: &str, main: &str, fallback: usize) -> Range<usize> {
    if let Some(r) = main.substr_range(sub) {
        r.start + origin..r.end + origin
    } else {
        fallback..fallback + 1
    }
}

impl Text3d {
    /// Parse rich text with no custom parsing functions, only standard styles are supported.
    ///
    #[doc = include_str!("parse_doc.md")]
    pub fn parse_raw(text: &str) -> Result<Self, ParseError<'_>> {
        Text3d::parse(text, ParseBuilder::new())
    }

    /// Parse rich text string.
    ///
    #[doc = include_str!("parse_doc.md")]
    pub fn parse(
        text: &str,
        parser: ParseBuilder<impl ParseStyleFn, impl ParseValueFn, impl ParseConditionFn>,
    ) -> Result<Self, ParseError<'_>> {
        match Text3d::parse_with_errors(text, parser) {
            (_result, Some(error)) => Err(error),
            (result, None) => Ok(result),
        }
    }

    /// Parse rich text with no custom parsing functions, only standard styles are supported.
    ///
    /// This function always produces a valid result and can capture owned strings, therefore easier to use with bsn.
    ///
    #[doc = include_str!("parse_doc.md")]
    pub fn bsn_parse_raw(text: impl AsRef<str>) -> Self {
        Text3d::parse_with_errors(text.as_ref(), ParseBuilder::new()).0
    }

    /// Parse rich text string.
    ///
    /// This function always produces a valid result and can capture owned strings, therefore easier to use with bsn.
    ///
    #[doc = include_str!("parse_doc.md")]
    pub fn bsn_parse(
        text: impl AsRef<str>,
        parser: ParseBuilder<impl ParseStyleFn, impl ParseValueFn, impl ParseConditionFn>,
    ) -> Self {
        Text3d::parse_with_errors(text.as_ref(), parser).0
    }
}

impl Text3d {
    /// Parse rich text string, always produces a result and returns the first error occurred.
    ///
    #[doc = include_str!("parse_doc.md")]
    pub fn parse_with_errors(
        text: &str,
        mut parser: ParseBuilder<impl ParseStyleFn, impl ParseValueFn, impl ParseConditionFn>,
    ) -> (Self, Option<ParseError<'_>>) {
        #[derive(Debug, Clone, Copy)]
        enum ParseState {
            Text,
            Command,
            Conditional(bool),
        }

        let style_backup = SegmentStyle::default();
        let mut buffer = String::new();
        let mut state = ParseState::Text;
        let mut segments = Vec::new();
        let mut stack: Vec<(SegmentStyle, Option<usize>)> = vec![(SegmentStyle::default(), None)];
        let mut error = OnceCell::new();
        let mut span_start = 0;
        let mut pos = 0;

        macro_rules! push_seg {
            () => {
                let style = style!();
                if !buffer.is_empty() {
                    segments.push((Text3dSegment::String(core::mem::take(&mut buffer)), style));
                }
            };
        }
        macro_rules! style {
            () => {
                stack
                    .last()
                    .map(|x| &x.0)
                    .unwrap_or_else(|| {
                        let _ = error.set(ParseError {
                            text,
                            span: (pos..pos + 1),
                            error: ParseErrorType::BracketMismatch,
                        });
                        &style_backup
                    })
                    .clone()
            };
            ($($tt: tt)*) => {
                {
                    let _ = stack
                        .last_mut()
                        .map(|x| &mut x.0)
                        .map($($tt)*);
                }
            };
        }
        use ParseState::*;
        let mut iter = text.char_indices().peekable();

        macro_rules! peek {
            () => {
                iter.peek().map(|x| x.1)
            };
        }

        while let Some((i, c)) = iter.next() {
            pos = i;
            match (c, state) {
                ('{', Text) => {
                    push_seg!();
                    span_start = i + 1;
                    state = Command;
                }
                (' ', Command) if buffer.is_empty() => {}
                ('?', Command) if buffer.is_empty() => {
                    span_start = i + 1;
                    state = Conditional(true);
                }
                ('!', Conditional(true)) if buffer.is_empty() => {
                    span_start = i + 1;
                    state = Conditional(false);
                }
                (':', Command) => {
                    let mut style = style!();
                    for s in buffer.trim().split(",") {
                        match parse_style(s.trim(), &mut parser.parse_style) {
                            Ok(s) => style = style.join(s),
                            Err(e) => {
                                let span = sub_span(span_start, s, &buffer, pos);
                                let _ = error.set(ParseError {
                                    text,
                                    span,
                                    error: ParseErrorType::StyleError(e),
                                });
                            }
                        }
                    }
                    stack.push((style, None));
                    buffer.clear();
                    state = Text;
                }
                (':', Conditional(should_be)) => {
                    let s = buffer.trim();
                    match parser.parse_condition.call(s) {
                        Ok(ConditionOutput::Constant(b)) if b == should_be => {
                            // start a scope and do nothing
                            stack.push((style!(), None));
                        }
                        Ok(ConditionOutput::Constant(_)) => {
                            // skip all wrapped items.
                            let mut depth = 1;
                            while depth > 0 {
                                match iter.next() {
                                    Some((_, '{')) => depth += 1,
                                    Some((_, '}')) => depth -= 1,
                                    _ => (),
                                }
                            }
                        }
                        Ok(ConditionOutput::Dynamic(entity)) => {
                            let pos = segments.len();
                            segments.push((
                                Text3dSegment::SkipIf {
                                    condition: entity,
                                    skip_if: !should_be,
                                    offset: 0,
                                },
                                style!(),
                            ));
                            stack.push((style!(), Some(pos)));
                        }
                        Err(e) => {
                            // start a scope and do nothing
                            stack.push((style!(), None));

                            let span = sub_span(span_start, s, &buffer, pos);
                            let _ = error.set(ParseError {
                                text,
                                span,
                                error: ParseErrorType::ConditionError(e),
                            });
                        }
                    }
                    buffer.clear();
                    state = Text;
                }
                ('}', Text) => {
                    trim_mut(&mut buffer);
                    push_seg!();
                    if let Some((_, Some(r))) = stack.pop() {
                        let l = segments.len().saturating_sub(1 + r);
                        if let Text3dSegment::SkipIf { offset, .. } = &mut segments[r].0 {
                            *offset = l;
                        }
                    };
                }
                ('}', Command) => {
                    let s = buffer.trim();
                    match parser.parse_value.call(segments.len(), s) {
                        Ok((segment, style)) => {
                            let style = style!().join(style);
                            segments.push((segment, style));
                        }
                        Err(e) => {
                            let span = sub_span(span_start, s, &buffer, pos);
                            let _ = error.set(ParseError {
                                text,
                                span,
                                error: ParseErrorType::ValueError(e),
                            });
                        }
                    }
                    buffer.clear();
                    state = Text;
                }
                ('}', Conditional(_)) => {} // do nothing, failing is too harsh.
                ('*', Text) => {
                    push_seg!();
                    let mut stars = 1;
                    while let Some(c) = peek!() {
                        if c == '*' {
                            stars += 1;
                            iter.next();
                        } else {
                            break;
                        }
                    }
                    match stars {
                        1 => style!(|s| s.style.flip()),
                        2 => style!(|s| s.weight.flip()),
                        3 => {
                            style!(|s| s.style.flip());
                            style!(|s| s.weight.flip());
                        }
                        n if n % 2 == 0 => (),
                        _ => style!(|s| s.style.flip()),
                    }
                }
                ('_', Text) if peek!() == Some('_') => {
                    push_seg!();
                    iter.next();
                    style!(|s| s.underline.flip());
                }
                ('~', Text) if peek!() == Some('~') => {
                    push_seg!();
                    iter.next();
                    style!(|s| s.strikethrough.flip());
                }
                (c, Command | Conditional(_)) => buffer.push(c),
                ('\\', Text) => {
                    if let Some(c) = peek!() {
                        buffer.push(c);
                        iter.next();
                    }
                }
                (c, Text) if c.is_whitespace() => {
                    let mut linebreaks = if c == '\n' { 1 } else { 0 };
                    while let Some(c) = peek!() {
                        if !c.is_whitespace() {
                            break;
                        } else if c == '\n' {
                            linebreaks += 1;
                        }
                        iter.next();
                    }
                    match linebreaks {
                        0 => buffer.push(' '),
                        n => buffer.extend(repeat_n('\n', n)),
                    }
                }
                (c, Text) => {
                    buffer.push(c);
                }
            }
        }
        push_seg!();
        (Text3d { segments }, error.take())
    }
}

fn parse_style(
    style: &str,
    stylesheet: &mut impl ParseStyleFn,
) -> Result<SegmentStyle, Option<String>> {
    if let Some(number) = style.strip_prefix("v-") {
        if let Ok(magic_number) = f32::from_str(number) {
            Ok(SegmentStyle {
                magic_number: Some(magic_number),
                ..Default::default()
            })
        } else {
            stylesheet.call(style)
        }
    } else if let Some(name) = style.strip_prefix("s-") {
        if let Ok(int) = u32::from_str(name) {
            Ok(SegmentStyle {
                stroke: NonZeroU32::new(int),
                ..Default::default()
            })
        } else if let Some(color) = parse_color(name) {
            Ok(SegmentStyle {
                stroke_color: Some(color),
                ..Default::default()
            })
        } else {
            stylesheet.call(style)
        }
    } else if let Some(name) = style.strip_prefix("f-") {
        Ok(SegmentStyle {
            font: Some(name.into()),
            ..Default::default()
        })
    } else if let Some(name) = style.strip_prefix("$") {
        if let Ok(size) = name.parse::<f32>() {
            Ok(SegmentStyle {
                size: Some(SegmentSize::Flat(size)),
                ..Default::default()
            })
        } else {
            stylesheet.call(style)
        }
    } else if let Some(name) = style.strip_prefix("*") {
        if let Ok(size) = name.parse::<f32>() {
            Ok(SegmentStyle {
                size: Some(SegmentSize::Multiply(size)),
                ..Default::default()
            })
        } else {
            stylesheet.call(style)
        }
    } else if let Some(color) = parse_color(style) {
        Ok(SegmentStyle {
            fill_color: Some(color),
            ..Default::default()
        })
    } else {
        match style {
            "bold" => Ok(SegmentStyle {
                weight: Some(Weight::BOLD),
                ..Default::default()
            }),
            "italic" => Ok(SegmentStyle {
                style: Some(Style::Italic),
                ..Default::default()
            }),
            "underline" => Ok(SegmentStyle {
                underline: Some(true),
                ..Default::default()
            }),
            "strikethrough" => Ok(SegmentStyle {
                strikethrough: Some(true),
                ..Default::default()
            }),
            "h1" => Ok(SegmentStyle {
                size: Some(SegmentSize::Multiply(2.0)),
                ..Default::default()
            }),
            "h2" => Ok(SegmentStyle {
                size: Some(SegmentSize::Multiply(1.75)),
                ..Default::default()
            }),
            "h3" => Ok(SegmentStyle {
                size: Some(SegmentSize::Multiply(1.5)),
                ..Default::default()
            }),
            "h4" => Ok(SegmentStyle {
                size: Some(SegmentSize::Multiply(1.25)),
                ..Default::default()
            }),
            _ => stylesheet.call(style),
        }
    }
}
