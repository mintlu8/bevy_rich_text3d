use std::{iter::repeat_n, num::NonZeroU32, str::FromStr};

use crate::{
    color_table::parse_color,
    misc::{Style, Weight},
    parse_util::{
        ConditionOutput, Flip, ParseBuilder, ParseConditionFn, ParseError, ParseStyleFn,
        ParseValueFn,
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

impl Text3d {
    /// Call [`Text3d::parse`] with no custom parsing functions.
    ///
    /// Only standard styles are supported, see [`Text3d::parse`] for details.
    pub fn parse_raw(text: &str) -> Result<Self, ParseError> {
        Text3d::parse(text, ParseBuilder::new())
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
    /// `value` will be trimmed of whitespace, so the correct syntax is to add whitespace outside of the braces:
    ///
    /// ```md
    /// Hello {style:World}! // Hello World!
    /// Hello{style: World}! // HelloWorld!
    /// ```
    ///
    /// Style commands also can be chained:
    ///
    /// ```md
    /// Deals {red, s-black, s-10: 10} damage!
    /// ```
    ///
    /// ## Standard Styles
    ///
    /// These will be parsed regardless of the `stylesheet` function:
    ///
    /// * `red` Parses Css color names as fill color.
    /// * `#ff00ff` Parses hex color (accepts 3, 4, 6, 8 digits) as fill color.
    /// * `s-4` Sets stroke to a number.
    /// * `s-red` Parses color names as stroke color.
    /// * `v-4.0` Sets the `magic_number` field.
    /// * `f-Roboto` Sets the font to Roboto.
    /// * `$18` Sets font size to `18`.
    /// * `*1.5` Sets font size to `1.5` times the original.
    /// * `h1` - `h4` Sets font size to `2`, `1.75`, `1.5`, `1.25` times the original.
    ///
    /// ## Dynamic value
    ///
    /// ```md
    /// { value }
    /// ```
    ///
    /// Without `:` values in brackets are treated as dynamic values and passed to the `fetch_string` function.
    /// The result should either be a string fetched from the world
    /// or an [`Entity`](bevy::ecs::entity::Entity) with a [`FetchedTextSegment`](crate::FetchedTextSegment) component.
    ///
    /// ## Conditions
    ///
    /// ```md
    /// { ?condition : value }
    /// { ?!condition : value }        // flips the condition
    /// ```
    ///
    /// Displays value only when `condition` is true (or false with `?!`).
    /// The result should either be a boolean value fetched from the world
    /// or an [`Entity`](bevy::ecs::entity::Entity) with a [`FetchedCondition`](crate::FetchedCondition) component.
    ///
    /// ## Markdown
    ///
    /// A subset of markdown features are supported:
    /// * `*emphasis*`
    /// * `**strong**`
    /// * `__underline__`
    /// * `~~strikethrough~~`
    /// * `\*` escape character
    ///
    /// ## Whitespace Rule
    ///
    /// Consecutive whitespaces are rendered either as one whitespace or multiple linebreaks.
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
        mut parser: ParseBuilder<impl ParseStyleFn, impl ParseValueFn, impl ParseConditionFn>,
    ) -> Result<Self, ParseError> {
        #[derive(Debug, Clone, Copy)]
        enum ParseState {
            Text,
            Command,
            Conditional(bool),
        }

        let mut buffer = String::new();
        let mut state = ParseState::Text;
        let mut segments = Vec::new();
        let mut stack: Vec<(SegmentStyle, Option<usize>)> = vec![(SegmentStyle::default(), None)];

        macro_rules! push_seg {
            () => {
                if !buffer.is_empty() {
                    segments.push((
                        Text3dSegment::String(core::mem::take(&mut buffer)),
                        style!(),
                    ));
                }
            };
        }
        macro_rules! style {
            () => {
                stack
                    .last()
                    .map(|x| &x.0)
                    .ok_or(ParseError::BracketMismatch)?
                    .clone()
            };
            (mut) => {
                stack
                    .last_mut()
                    .map(|x| &mut x.0)
                    .ok_or(ParseError::BracketMismatch)?
            };
        }
        use ParseState::*;
        let mut iter = text.chars().peekable();
        while let Some(c) = iter.next() {
            match (c, state) {
                ('{', Text) => {
                    push_seg!();
                    state = Command;
                }
                (' ', Command) if buffer.is_empty() => {}
                ('?', Command) if buffer.is_empty() => {
                    state = Conditional(true);
                }
                ('!', Conditional(true)) if buffer.is_empty() => {
                    state = Conditional(false);
                }
                (':', Command) => {
                    let mut style = style!();
                    for s in buffer.trim().split(",") {
                        style = style.join(parse_style(s.trim(), &mut parser.parse_style)?)
                    }
                    stack.push((style, None));
                    buffer.clear();
                    state = Text;
                }
                (':', Conditional(should_be)) => {
                    match parser.parse_condition.call(buffer.trim())? {
                        ConditionOutput::Constant(b) if b == should_be => {
                            // start a scope and do nothing
                            stack.push((style!(), None));
                        }
                        ConditionOutput::Constant(_) => {
                            // skip all wrapped items.
                            let mut depth = 1;
                            while depth > 0 {
                                match iter.next() {
                                    Some('{') => depth += 1,
                                    Some('}') => depth -= 1,
                                    _ => (),
                                }
                            }
                        }
                        ConditionOutput::Dynamic(entity) => {
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
                    let (segment, style) = parser.parse_value.call(buffer.trim())?;
                    let style = style!().join(style);
                    segments.push((segment, style));
                    buffer.clear();
                    state = Text;
                }
                ('}', Conditional(_)) => {} // do nothing, failing is too harsh.
                ('*', Text) => {
                    push_seg!();
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
                ('_', Text) if iter.peek() == Some(&'_') => {
                    push_seg!();
                    iter.next();
                    style!(mut).underline.flip()
                }
                ('~', Text) if iter.peek() == Some(&'~') => {
                    push_seg!();
                    iter.next();
                    style!(mut).strikethrough.flip()
                }
                (c, Command | Conditional(_)) => buffer.push(c),
                ('\\', Text) => {
                    if let Some(c) = iter.peek() {
                        buffer.push(*c);
                        iter.next();
                    } else {
                        buffer.push('\\');
                    }
                }
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
        push_seg!();
        Ok(Text3d { segments })
    }
}

fn parse_style(
    style: &str,
    stylesheet: &mut impl ParseStyleFn,
) -> Result<SegmentStyle, ParseError> {
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
