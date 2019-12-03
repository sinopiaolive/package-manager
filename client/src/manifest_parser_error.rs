use failure;
use pest;
use std::convert::From;
use std::fmt::Display;

use crate::manifest_parser::Rule;

// We'd like to store original Pest errors in this error type, but Pest errors
// don't implement Send (due to Rc fields), which the Fail trait requires. So as
// a workaround, we stringify them on instantiation.
#[derive(Fail, Debug)]
pub enum ManifestParserError {
    #[fail(display = "{}", _0)]
    PestError(String),
    #[fail(display = "{}", description)]
    ErrorAtSpan {
        description: String,
        // We'd like to tag the original_error as #[cause], but failure::Error
        // doesn't implement std::error::Error (yet?).
        original_error: failure::Error,
    },
    #[fail(display = "{}", description)]
    ErrorAtPos {
        description: String,
        original_error: failure::Error,
    },
}

pub trait PestErrorExt<E>
where
    failure::Error: From<E>,
    E: Display,
{
    fn with_span<I>(self, span: &pest::inputs::Span<I>) -> ManifestParserError
    where
        I: pest::inputs::Input;

    fn with_pair<R, I>(self, pair: &pest::iterators::Pair<R, I>) -> ManifestParserError
    where
        R: pest::RuleType,
        I: pest::inputs::Input;

    fn with_pos<I>(self, pos: &pest::inputs::Position<I>) -> ManifestParserError
    where
        I: pest::inputs::Input;
}

impl<E> PestErrorExt<E> for E
where
    failure::Error: From<E>,
    E: Display,
{
    fn with_span<I>(self, span: &pest::inputs::Span<I>) -> ManifestParserError
    where
        I: pest::inputs::Input,
    {
        let dummy_pest_error: pest::Error<Rule, I> = pest::Error::CustomErrorSpan {
            message: format!("{}", &self),
            span: span.clone(),
        };
        ManifestParserError::ErrorAtSpan {
            description: format!("{}", dummy_pest_error),
            original_error: failure::Error::from(self),
        }
    }

    fn with_pair<R, I>(self, pair: &pest::iterators::Pair<R, I>) -> ManifestParserError
    where
        R: pest::RuleType,
        I: pest::inputs::Input,
    {
        self.with_span(&pair.clone().into_span())
    }

    fn with_pos<I>(self, pos: &pest::inputs::Position<I>) -> ManifestParserError
    where
        I: pest::inputs::Input,
    {
        let dummy_pest_error: pest::Error<Rule, I> = pest::Error::CustomErrorPos {
            message: format!("{}", &self),
            pos: pos.clone(),
        };
        ManifestParserError::ErrorAtPos {
            description: format!("{}", dummy_pest_error),
            original_error: failure::Error::from(self),
        }
    }
}

pub trait PestResultExt<T, E>
where
    failure::Error: From<E>,
    E: Display,
{
    fn span_context<I>(self, span: &pest::inputs::Span<I>) -> Result<T, ManifestParserError>
    where
        I: pest::inputs::Input;

    fn pair_context<R, I>(
        self,
        pair: &pest::iterators::Pair<R, I>,
    ) -> Result<T, ManifestParserError>
    where
        R: pest::RuleType,
        I: pest::inputs::Input;

    fn pos_context<I>(self, pos: &pest::inputs::Position<I>) -> Result<T, ManifestParserError>
    where
        I: pest::inputs::Input;
}

impl<T, E> PestResultExt<T, E> for Result<T, E>
where
    failure::Error: From<E>,
    E: Display,
{
    fn span_context<I>(self, span: &pest::inputs::Span<I>) -> Result<T, ManifestParserError>
    where
        I: pest::inputs::Input,
    {
        self.map_err(|err| err.with_span(span))
    }

    fn pair_context<R, I>(
        self,
        pair: &pest::iterators::Pair<R, I>,
    ) -> Result<T, ManifestParserError>
    where
        R: pest::RuleType,
        I: pest::inputs::Input,
    {
        self.map_err(|err| err.with_pair(pair))
    }

    fn pos_context<I>(self, pos: &pest::inputs::Position<I>) -> Result<T, ManifestParserError>
    where
        I: pest::inputs::Input,
    {
        self.map_err(|err| err.with_pos(pos))
    }
}

impl From<pest::Error<Rule, pest::inputs::StringInput>> for ManifestParserError {
    fn from(pest_error: pest::Error<Rule, pest::inputs::StringInput>) -> Self {
        ManifestParserError::PestError(format!("{}", pest_error))
    }
}
