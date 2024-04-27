//! <https://html.spec.whatwg.org/multipage/links.html>

/// Describes the content linked by a `<link>` element
///
/// [Specification](https://html.spec.whatwg.org/multipage/links.html#linkTypes)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Relationship {
    /// <https://html.spec.whatwg.org/multipage/links.html#rel-alternate>
    Alternate,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-canonical>
    Canonical,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-author>
    Author,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-bookmark>
    Bookmark,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-dns-prefetch>
    DnsPrefetch,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-expect>
    Expect,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-external>
    External,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-help>
    Help,

    /// <https://html.spec.whatwg.org/multipage/links.html#rel-icon>
    Icon,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-manifest>
    Manifest,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-modulepreload>
    ModulePreload,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-license>
    License,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-next>
    Next,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-nofollow>
    NoFollow,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-noopener>
    NoOpener,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-noreferrer>
    NoReferrer,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-opener>
    Opener,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-pingback>
    PingBack,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-preconnect>
    PreConnect,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-prefetch>
    PreFetch,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-preload>
    PreLoad,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-prev>
    Prev,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-privacy-policy>
    PrivacyPolicy,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-search>
    Search,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-stylesheet>
    Stylesheet,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-tag>
    Tag,

    /// <https://html.spec.whatwg.org/multipage/links.html#link-type-terms-of-service>
    TermsOfService,

    /// The `rel` attribute was either not given or could not be understood
    #[default]
    Invalid,
}

impl From<&str> for Relationship {
    fn from(value: &str) -> Self {
        match value {
            "alternate" => Self::Alternate,
            "canonical" => Self::Canonical,
            "author" => Self::Author,
            "bookmark" => Self::Bookmark,
            "dns-prefetch" => Self::DnsPrefetch,
            "expect" => Self::Expect,
            "external" => Self::External,
            "help" => Self::Help,
            "icon" => Self::Icon,
            "manifest" => Self::Manifest,
            "modulepreload" => Self::ModulePreload,
            "license" => Self::License,
            "next" => Self::Next,
            "nofollow" => Self::NoFollow,
            "noopener" => Self::NoOpener,
            "noreferrer" => Self::NoReferrer,
            "opener" => Self::Opener,
            "pingback" => Self::PingBack,
            "preconnect" => Self::PreConnect,
            "prefetch" => Self::PreFetch,
            "preload" => Self::PreLoad,
            "prev" => Self::Prev,
            "privacy-policy" => Self::PrivacyPolicy,
            "search" => Self::Search,
            "stylesheet" => Self::Stylesheet,
            "tag" => Self::Tag,
            "terms-of-service" => Self::TermsOfService,
            _ => Self::Invalid,
        }
    }
}
