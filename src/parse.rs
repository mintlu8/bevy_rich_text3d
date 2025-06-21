use std::{iter::repeat_n, num::NonZeroU32, str::FromStr};

use cosmic_text::{Style, Weight};

use crate::{color_table::parse_color, SegmentStyle, Text3d, Text3dSegment};

trait Flip {
    fn flip(&mut self);
}

impl Flip for Option<Weight> {
    fn flip(&mut self) {
        *self = match *self {
            Some(w) if w <= Weight::NORMAL => Some(Weight::BOLD),
            None => Some(Weight::BOLD),
            _ => Some(Weight::NORMAL),
        }
    }
}

impl Flip for Option<Style> {
    fn flip(&mut self) {
        *self = match *self {
            Some(Style::Normal) | None => Some(Style::Italic),
            _ => Some(Style::Italic),
        }
    }
}

impl Text3d {
    /// Call [`Text3d::parse`] with no custom parsing functions.
    ///
    /// Only standard styles are supported, see [`Text3d::parse`] for details.
    pub fn parse_raw(text: &str) -> Result<Self, ParseError> {
        Text3d::parse(
            text,
            |command| Err(ParseError::BadCommand(command.into())),
            |style| Err(ParseError::MissingStyle(style.into())),
        )
    }

    /// Parse rich text string.
    ///
    /// # Example
    ///
    /// ```
    /// "Deals **{blue:{damage_number}}** {red:fire} damage to the enemy."
    /// ```
    ///
    /// # Syntax
    ///
    /// ## Style
    ///
    /// ```md
    /// {style:value}
    /// ```
    ///
    /// This is equivalent to `<style>value</style>` in html.
    /// The left hand side is the name of the style, it will be passed to the `stylesheet` function.
    ///
    /// Style commands also can be chained:
    ///
    /// ```md
    /// Deals {red, s-black, s-10: 10} damage!
    /// ```
    ///
    /// ## Standard Styles
    ///
    /// These will be parsed without the `stylesheet` function:
    ///
    /// * `red` Parses Css color names as fill color.
    /// * `#ff00ff` Parses hex color (accepts 3, 4, 6, 8 digits) as fill color.
    /// * `s-4` Sets stroke to a number.
    /// * `s-red` Parses color names as stroke color.
    /// * `v-4.0` Sets the `magic_number` field.
    ///
    /// ## Dynamic value
    ///
    /// ```md
    /// { value }
    /// ```
    ///
    /// Without `:` values in brackets are treated as dynamic values and passed to the `fetch_string` function.
    /// The result should either be a string fetched from the world
    /// or an [`Entity`](bevy::prelude::Entity) with a [`FetchedTextSegment`](crate::FetchedTextSegment) component.
    ///
    ///
    /// ## Markdown
    ///
    /// A subset of markdown features are supported:
    /// * `*emphasis*`
    /// * `**strong**`
    ///
    /// ## Inputs
    ///
    /// * `fetch_string`: Parses strings to obtain values from the world.
    ///     * [`Text3dSegment::String`] should be returned for static values.
    ///     * [`Text3dSegment::Extract`] should be returned after spawning a string fetcher for dynamic values.
    /// * `stylesheet`: Parses strings as [`SegmentStyle`].
    ///
    /// We trim whitespaces before passing arguments to these functions.
    pub fn parse(
        text: &str,
        mut fetch_string: impl FnMut(&str) -> Result<Text3dSegment, ParseError>,
        mut stylesheet: impl FnMut(&str) -> Result<SegmentStyle, ParseError>,
    ) -> Result<Self, ParseError> {
        #[derive(Debug, Clone, Copy)]
        enum ParseState {
            Text,
            Command,
            Image,
        }

        trait BooleanFlip {
            fn flip(&mut self);
        }

        impl BooleanFlip for bool {
            fn flip(&mut self) {
                *self = !*self;
            }
        }

        let mut buffer = String::new();
        let mut state = ParseState::Text;
        let mut segments = Vec::new();
        let mut styles = vec![SegmentStyle::default()];
        macro_rules! style {
            () => {
                styles.last().ok_or(ParseError::BracketMismatch)?
            };
            (mut) => {
                styles.last_mut().ok_or(ParseError::BracketMismatch)?
            };
        }
        use ParseState::*;
        let mut iter = text.chars().peekable();
        while let Some(c) = iter.next() {
            match (c, state) {
                ('{', Text) => {
                    push_segment(&buffer, &mut segments, &mut styles)?;
                    buffer.clear();
                    state = Command;
                }
                (':', Command) => match buffer.trim().split(",").collect::<Vec<_>>().as_slice() {
                    ["image"] => {
                        buffer.clear();
                        state = Image;
                    }
                    style_slice => {
                        let mut style = style!().clone();
                        for s in style_slice {
                            style = style.join(parse_style(s.trim(), &mut stylesheet)?)
                        }
                        styles.push(style);
                        buffer.clear();
                        state = Text;
                    }
                },
                ('}', Text) => {
                    push_segment(&buffer, &mut segments, &mut styles)?;
                    buffer.clear();
                    let _ = styles.pop();
                }
                ('}', Command) => {
                    segments.push((fetch_string(buffer.trim())?, style!().clone()));
                    buffer.clear();
                    state = Text;
                }
                ('}', Image) => {
                    return Err(ParseError::NotSupported("image"));
                }
                ('*', Text) => {
                    push_segment(&buffer, &mut segments, &mut styles)?;
                    buffer.clear();
                    let mut stars = 1;
                    while let Some(c) = iter.peek() {
                        if *c == '*' {
                            stars += 1;
                            iter.next();
                        } else {
                            break;
                        }
                    }
                    match stars {
                        1 => style!(mut).style.flip(),
                        2 => style!(mut).weight.flip(),
                        3 => {
                            style!(mut).style.flip();
                            style!(mut).weight.flip();
                        }
                        n if n % 2 == 0 => (),
                        _ => style!(mut).style.flip(),
                    }
                }
                (c, Command | Image) => buffer.push(c),
                (c, Text) if c.is_whitespace() => {
                    let mut linebreaks = if c == '\n' { 1 } else { 0 };
                    while let Some(c) = iter.peek() {
                        if !c.is_whitespace() {
                            break;
                        } else if *c == '\n' {
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
        push_segment(&buffer, &mut segments, &mut styles)?;
        Ok(Text3d { segments })
    }
}

fn parse_style(
    style: &str,
    mut stylesheet: impl FnMut(&str) -> Result<SegmentStyle, ParseError>,
) -> Result<SegmentStyle, ParseError> {
    if style.starts_with("v-") {
        if let Ok(magic_number) = f32::from_str(style.split_at(2).1) {
            Ok(SegmentStyle {
                magic_number: Some(magic_number),
                ..Default::default()
            })
        } else {
            stylesheet(style)
        }
    } else if style.starts_with("s-") {
        if let Ok(int) = u32::from_str(style.split_at(2).1) {
            Ok(SegmentStyle {
                stroke: NonZeroU32::new(int),
                ..Default::default()
            })
        } else if let Some(color) = parse_color(style.split_at(2).1) {
            Ok(SegmentStyle {
                stroke_color: Some(color),
                ..Default::default()
            })
        } else {
            stylesheet(style)
        }
    } else if style.starts_with("bg-") {
        if let Some(color) = parse_color(style.split_at(3).1) {
            Ok(SegmentStyle {
                background_color: Some(color),
                ..Default::default()
            })
        } else {
            stylesheet(style)
        }
    } else if let Some(color) = parse_color(style) {
        Ok(SegmentStyle {
            fill_color: Some(color),
            ..Default::default()
        })
    } else {
        stylesheet(style)
    }
}

fn push_segment(
    buffer: &str,
    spans: &mut Vec<(Text3dSegment, SegmentStyle)>,
    styles: &mut [SegmentStyle],
) -> Result<(), ParseError> {
    if !buffer.is_empty() {
        spans.push((
            Text3dSegment::String(buffer.into()),
            styles.last().ok_or(ParseError::BracketMismatch)?.clone(),
        ));
    }
    Ok(())
}

/// Error emitted when parsing rich text.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Feature {0} is not supported.")]
    NotSupported(&'static str),
    #[error("Bracket mismatch.")]
    BracketMismatch,
    #[error("Bad command: {0}")]
    BadCommand(String),
    #[error("Style {0} missing.")]
    MissingStyle(String),
    #[error("{0}")]
    Custom(String),
}
