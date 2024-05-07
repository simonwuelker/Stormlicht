//! Utilites to understand the value of the `cache-control` header

use std::iter::FusedIterator;

use super::utils::{CommaSeparatedHeader, HeaderDirective};

/// Different possible directives of the `cache-control` header
///
/// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control#response_directives>
/// for more information
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheControlDirective {
    MaxAge(usize),
    MaxStale(usize),
    MinFresh(usize),
    SMaxAge(usize),
    NoCache,
    NoStore,
    NoTransform,
    OnlyIfCached,
    MustRevalidate,
    ProxyRevalidate,
    MustUnderstand,
    Private,
    Public,
    Immutable,
    StaleWhileRevalidate,
}

#[derive(Clone, Copy, Debug)]
pub enum CacheControlParseError<'a> {
    UnknownDirective(&'a str),
    BadValueForDirective { directive: &'a str, value: &'a str },
}

#[derive(Clone, Debug)]
pub struct CacheControlIterator {
    internal: CommaSeparatedHeader,
}

impl CacheControlIterator {
    /// An iterator that will never return any [CacheControlDirectives](CacheControlDirective)
    pub const EMPTY: Self = Self {
        internal: CommaSeparatedHeader::EMPTY,
    };

    #[must_use]
    pub fn new(header_value: &str) -> Self {
        Self {
            internal: CommaSeparatedHeader::new(header_value),
        }
    }
}

impl Iterator for CacheControlIterator {
    type Item = CacheControlDirective;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next_directive = self.internal.next_directive()?;

            match CacheControlDirective::try_from(next_directive) {
                Ok(cache_control) => return Some(cache_control),
                Err(error) => {
                    log::warn!("Failed to parse cache-control directive: {error:?}");
                },
            }
        }
    }
}

impl FusedIterator for CacheControlIterator {}

impl<'a> TryFrom<HeaderDirective<'a>> for CacheControlDirective {
    type Error = CacheControlParseError<'a>;

    fn try_from(directive: HeaderDirective<'a>) -> Result<Self, Self::Error> {
        let cache_control = match directive {
            HeaderDirective::KeyValuePair { key, value } => {
                // All cache control key-value pairs require a duration (seconds)
                let duration =
                    value
                        .parse()
                        .map_err(|_| CacheControlParseError::BadValueForDirective {
                            directive: key,
                            value,
                        })?;

                match key {
                    "max-age" => Self::MaxAge(duration),
                    "s-maxage" => Self::SMaxAge(duration),
                    "max-stale" => Self::MaxStale(duration),
                    "min-fresh" => Self::MinFresh(duration),
                    _ => {
                        return Err(CacheControlParseError::BadValueForDirective {
                            directive: key,
                            value,
                        })
                    },
                }
            },
            HeaderDirective::Value(directive) => match directive {
                "no-cache" => Self::NoCache,
                "no-store" => Self::NoStore,
                "no-transform" => Self::NoTransform,
                "only-if-cached" => Self::OnlyIfCached,
                "must-revalidate" => Self::MustRevalidate,
                "proxy-revalidate" => Self::ProxyRevalidate,
                "must-understand" => Self::MustUnderstand,
                "private" => Self::Private,
                "public" => Self::Public,
                "immutable" => Self::Immutable,
                "stale-while-revalidate" => Self::StaleWhileRevalidate,
                _ => return Err(CacheControlParseError::UnknownDirective(directive)),
            },
        };

        Ok(cache_control)
    }
}
