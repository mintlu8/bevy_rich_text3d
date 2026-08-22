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

/// Output for parsing condition.
///
/// Constant may skip a portion of the text entirely while dynamic checks [`FetchedCondition`](crate::FetchedCondition) at runtime.
#[derive(Debug, Clone, Copy)]
pub enum ConditionOutput {
    Constant(bool),
    Dynamic(Entity),
}

/// Placeholder default value.
pub struct DefaultFn;

pub trait ParseStyleFn {
    fn call(&mut self, s: &str) -> Result<SegmentStyle, ParseError>;
}

pub trait ParseValueFn {
    fn call(&mut self, index: usize, s: &str) -> Result<(Text3dSegment, SegmentStyle), ParseError>;
}

pub trait ParseConditionFn {
    fn call(&mut self, s: &str) -> Result<ConditionOutput, ParseError>;
}

impl ParseStyleFn for DefaultFn {
    fn call(&mut self, s: &str) -> Result<SegmentStyle, ParseError> {
        Err(ParseError::Custom(format!("Unknown style {s}.")))
    }
}

impl ParseValueFn for DefaultFn {
    fn call(
        &mut self,
        _index: usize,
        s: &str,
    ) -> Result<(Text3dSegment, SegmentStyle), ParseError> {
        Err(ParseError::Custom(format!("Unknown value {s}.")))
    }
}

impl ParseConditionFn for DefaultFn {
    fn call(&mut self, s: &str) -> Result<ConditionOutput, ParseError> {
        Err(ParseError::Custom(format!("Unknown condition {s}.")))
    }
}

impl<T: FnMut(&str) -> Result<SegmentStyle, ParseError>> ParseStyleFn for T {
    fn call(&mut self, s: &str) -> Result<SegmentStyle, ParseError> {
        self(s)
    }
}

impl<T: FnMut(&str) -> Result<(Text3dSegment, SegmentStyle), ParseError>> ParseValueFn for T {
    fn call(
        &mut self,
        _index: usize,
        s: &str,
    ) -> Result<(Text3dSegment, SegmentStyle), ParseError> {
        self(s)
    }
}

impl<T: FnMut(&str) -> Result<ConditionOutput, ParseError>> ParseConditionFn for T {
    fn call(&mut self, s: &str) -> Result<ConditionOutput, ParseError> {
        self(s)
    }
}

/// Builder pattern input for parsing rich text.
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
    pub fn with_parse_style<F: FnMut(&str) -> Result<SegmentStyle, ParseError>>(
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
    pub fn with_parse_value<F: FnMut(&str) -> Result<(Text3dSegment, SegmentStyle), ParseError>>(
        self,
        f: F,
    ) -> ParseBuilder<A, F, C> {
        ParseBuilder {
            parse_style: self.parse_style,
            parse_value: f,
            parse_condition: self.parse_condition,
        }
    }
}

impl<A: ParseStyleFn, B: ParseValueFn> ParseBuilder<A, B, DefaultFn> {
    pub fn with_parse_condition<F: FnMut(&str) -> Result<ConditionOutput, ParseError>>(
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
