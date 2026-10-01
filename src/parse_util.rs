use std::{
    fmt::{Display, Write},
    ops::Range,
};

use bevy::ecs::entity::Entity;

use crate::{SegmentStyle, Style, Text3dSegment, Weight};

pub(crate) trait Flip {
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

impl Flip for Option<bool> {
    fn flip(&mut self) {
        *self = match *self {
            Some(false) | None => Some(true),
            Some(true) => Some(false),
        }
    }
}

/// Error emitted when parsing rich text.
#[derive(Debug)]
pub enum ParseErrorType {
    BracketMismatch,
    StyleError(Option<String>),
    ValueError(Option<String>),
    ConditionError(Option<String>),
}

/// Error emitted when parsing rich text.
#[derive(Debug)]
pub struct ParseError<'t> {
    pub(crate) text: &'t str,
    pub(crate) span: Range<usize>,
    pub(crate) error: ParseErrorType,
}

/// Output for parsing condition.
///
/// Constant may skip a portion of the text entirely while dynamic checks [`FetchedCondition`](crate::FetchedCondition) at runtime.
#[derive(Debug, Clone, Copy)]
pub enum ConditionOutput {
    Constant(bool),
    Dynamic(Entity),
}

/// Placeholder default value.
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultFn;

pub trait ParseStyleFn {
    fn call(&mut self, s: &str) -> Result<SegmentStyle, Option<String>>;
}

pub trait ParseValueFn {
    fn call(
        &mut self,
        index: usize,
        s: &str,
    ) -> Result<(Text3dSegment, SegmentStyle), Option<String>>;
}

pub trait ParseConditionFn {
    fn call(&mut self, s: &str) -> Result<ConditionOutput, Option<String>>;
}

impl ParseStyleFn for DefaultFn {
    fn call(&mut self, _s: &str) -> Result<SegmentStyle, Option<String>> {
        Err(None)
    }
}

impl ParseValueFn for DefaultFn {
    fn call(
        &mut self,
        _index: usize,
        _s: &str,
    ) -> Result<(Text3dSegment, SegmentStyle), Option<String>> {
        Err(None)
    }
}

impl ParseConditionFn for DefaultFn {
    fn call(&mut self, _s: &str) -> Result<ConditionOutput, Option<String>> {
        Err(None)
    }
}

impl<T: FnMut(&str) -> Result<SegmentStyle, Option<String>>> ParseStyleFn for T {
    fn call(&mut self, s: &str) -> Result<SegmentStyle, Option<String>> {
        self(s)
    }
}

impl<T: FnMut(&str) -> Result<(Text3dSegment, SegmentStyle), Option<String>>> ParseValueFn for T {
    fn call(
        &mut self,
        _index: usize,
        s: &str,
    ) -> Result<(Text3dSegment, SegmentStyle), Option<String>> {
        self(s)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IndexedPVF<T>(T);

impl<T: FnMut(usize, &str) -> Result<(Text3dSegment, SegmentStyle), Option<String>>> ParseValueFn
    for IndexedPVF<T>
{
    fn call(
        &mut self,
        index: usize,
        s: &str,
    ) -> Result<(Text3dSegment, SegmentStyle), Option<String>> {
        self.0(index, s)
    }
}

impl<T: FnMut(&str) -> Result<ConditionOutput, Option<String>>> ParseConditionFn for T {
    fn call(&mut self, s: &str) -> Result<ConditionOutput, Option<String>> {
        self(s)
    }
}

/// Builder pattern input for parsing rich text.
#[derive(Debug, Clone, Copy)]
pub struct ParseBuilder<
    Style: ParseStyleFn = DefaultFn,
    Value: ParseValueFn = DefaultFn,
    Condition: ParseConditionFn = DefaultFn,
> {
    pub(crate) parse_style: Style,
    pub(crate) parse_value: Value,
    pub(crate) parse_condition: Condition,
}

impl Default for ParseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ParseBuilder {
    pub const fn new() -> Self {
        Self {
            parse_style: DefaultFn,
            parse_value: DefaultFn,
            parse_condition: DefaultFn,
        }
    }
}

impl<B: ParseValueFn, C: ParseConditionFn> ParseBuilder<DefaultFn, B, C> {
    pub fn with_parse_style<F: FnMut(&str) -> Result<SegmentStyle, Option<String>>>(
        self,
        f: F,
    ) -> ParseBuilder<F, B, C> {
        ParseBuilder {
            parse_style: f,
            parse_value: self.parse_value,
            parse_condition: self.parse_condition,
        }
    }
}

impl<A: ParseStyleFn, C: ParseConditionFn> ParseBuilder<A, DefaultFn, C> {
    pub fn with_parse_value<
        F: FnMut(&str) -> Result<(Text3dSegment, SegmentStyle), Option<String>>,
    >(
        self,
        f: F,
    ) -> ParseBuilder<A, F, C> {
        ParseBuilder {
            parse_style: self.parse_style,
            parse_value: f,
            parse_condition: self.parse_condition,
        }
    }

    pub fn with_parse_value_indexed<
        F: FnMut(usize, &str) -> Result<(Text3dSegment, SegmentStyle), Option<String>>,
    >(
        self,
        f: F,
    ) -> ParseBuilder<A, IndexedPVF<F>, C> {
        ParseBuilder {
            parse_style: self.parse_style,
            parse_value: IndexedPVF(f),
            parse_condition: self.parse_condition,
        }
    }
}

impl<A: ParseStyleFn, B: ParseValueFn> ParseBuilder<A, B, DefaultFn> {
    pub fn with_parse_condition<F: FnMut(&str) -> Result<ConditionOutput, Option<String>>>(
        self,
        f: F,
    ) -> ParseBuilder<A, B, F> {
        ParseBuilder {
            parse_style: self.parse_style,
            parse_value: self.parse_value,
            parse_condition: f,
        }
    }
}

impl Display for ParseError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        pub use ParseErrorType::*;
        f.write_str("ParseError: ")?;
        let substr = self.text.get(self.span.clone()).unwrap_or("");
        match &self.error {
            BracketMismatch => f.write_str("bracket mismatch")?,
            StyleError(None) => write!(f, "invalid style: `{}`", substr)?,
            ValueError(None) => write!(f, "invalid value: `{}`", substr)?,
            ConditionError(None) => write!(f, "invalid condition: `{}`", substr)?,
            StyleError(Some(message)) => {
                write!(f, "error parsing style `{}`: {}", substr, message)?
            }
            ValueError(Some(message)) => {
                write!(f, "error parsing value `{}`: {}", substr, message)?
            }
            ConditionError(Some(message)) => {
                write!(f, "error parsing condition `{}`: {}", substr, message)?
            }
        }
        let mut start = 0;
        let mut line_number = 0;
        for (idx, char) in self.text.char_indices() {
            if idx >= self.span.start {
                break;
            }
            if char == '\n' {
                start = idx + 1;
                line_number += 1;
            }
        }
        let (_, text) = self.text.split_at(start);

        write!(f, "\n   |\n{line_number:<3}| ")?;
        let mut buf2 = "   | ".to_owned();

        for (idx, char) in text.char_indices() {
            let idx = idx + start;
            if char == '\n' {
                line_number += 1;
                writeln!(f, "\n{}", buf2)?;
                buf2.truncate(5);
                if idx >= self.span.end {
                    return Ok(());
                } else {
                    write!(f, "{line_number:<3}|")?;
                }
            } else {
                f.write_char(char)?;
                if self.span.contains(&idx) {
                    buf2.push('^');
                } else {
                    buf2.push(' ');
                }
            }
        }
        writeln!(f, "\n{}", buf2)?;
        Ok(())
    }
}
