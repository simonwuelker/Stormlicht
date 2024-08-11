//! [HTTP status codes](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status)

use std::{fmt, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StatusCode(u16);

impl StatusCode {
    // information
    pub const CONTINUE: Self = Self(100);
    pub const SWITCHING_PROTOCOLS: Self = Self(101);
    pub const PROCESSING: Self = Self(102);
    pub const EARLY_HINTS: Self = Self(103);

    // success
    pub const OK: Self = Self(200);
    pub const CREATED: Self = Self(201);
    pub const ACCEPTED: Self = Self(202);
    pub const NON_AUTHORITATIVE_INFORMATION: Self = Self(203);
    pub const NO_CONTENT: Self = Self(204);
    pub const RESET_CONTENT: Self = Self(205);
    pub const PARTIAL_CONTENT: Self = Self(206);
    pub const MULTI_STATUS: Self = Self(207);
    pub const ALREADY_REPORTED: Self = Self(208);
    pub const IM_USED: Self = Self(226);

    // redirection
    pub const MULTIPLE_CHOICES: Self = Self(300);
    pub const MOVED_PERMANENTLY: Self = Self(301);
    pub const FOUND: Self = Self(302);
    pub const SEE_OTHER: Self = Self(303);
    pub const NOT_MODIFIED: Self = Self(304);
    pub const USE_PROXY: Self = Self(305);
    pub const UNUSED: Self = Self(306); // reserved status code
    pub const TEMPORARY_REDIRECT: Self = Self(307);
    pub const PERMANENT_REDIRECT: Self = Self(308);

    // client error
    pub const BAD_REQUEST: Self = Self(400);
    pub const UNAUTHORIZED: Self = Self(401);
    pub const PAYMENT_REQUIRED: Self = Self(402);
    pub const FORBIDDEN: Self = Self(403);
    pub const NOT_FOUND: Self = Self(404);
    pub const METHOD_NOT_ALLOWED: Self = Self(405);
    pub const NOT_ACCEPTABLE: Self = Self(406);
    pub const PROXY_AUTHENTICATION_REQUIRED: Self = Self(407);
    pub const REQUEST_TIMEOUT: Self = Self(408);
    pub const CONFLICT: Self = Self(409);
    pub const GONE: Self = Self(410);
    pub const LENGTH_REQUIRED: Self = Self(411);
    pub const PRECONDITION_FAILED: Self = Self(412);
    pub const PAYLOAD_TOO_LARGE: Self = Self(413);
    pub const URI_TOO_LONG: Self = Self(414);
    pub const UNSUPPORTED_MEDIA_TYPE: Self = Self(415);
    pub const RANGE_NOT_SATISFIABLE: Self = Self(416);
    pub const EXPECTATION_FAILED: Self = Self(417);
    pub const IM_A_TEAPOT: Self = Self(418);
    pub const MISDIRECTED_REQUEST: Self = Self(421);
    pub const UNPROCESSABLE_ENTITY: Self = Self(422);
    pub const LOCKED: Self = Self(423);
    pub const FAILED_DEPENDENCY: Self = Self(424);
    pub const TOO_EARLY: Self = Self(425);
    pub const UPGRADE_REQUIRED: Self = Self(426);
    pub const PRECONDITION_REQUIRED: Self = Self(428);
    pub const TOO_MANY_REQUESTS: Self = Self(429);
    pub const REQUEST_HEADER_FIELD_TOO_LARGE: Self = Self(431);
    pub const UNAVAILABLE_FOR_LEGAL_REASONS: Self = Self(451);

    // server error
    pub const INTERNAL_SERVER_ERROR: Self = Self(500);
    pub const NOT_IMPLEMENTED: Self = Self(501);
    pub const BAD_GATEWAY: Self = Self(502);
    pub const SERVICE_UNAVAILABLE: Self = Self(503);
    pub const GATEWAY_TIMEOUT: Self = Self(504);
    pub const HTTP_VERSION_NOT_SUPPORTED: Self = Self(505);
    pub const VARIANT_ALSO_NEGOTIATES: Self = Self(506);
    pub const INSUFFICIENT_STORAGE: Self = Self(507);
    pub const LOOP_DETECTED: Self = Self(508);
    pub const NOT_EXTENDED: Self = Self(510);
    pub const NETWORK_AUTHENTICATION_REQUIRED: Self = Self(511);

    #[must_use]
    pub const fn numeric(&self) -> u16 {
        self.0
    }

    #[must_use]
    pub const fn is_informational(&self) -> bool {
        matches!(self.numeric(), 100..200)
    }

    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self.numeric(), 200..300)
    }

    #[must_use]
    pub const fn is_redirection(&self) -> bool {
        matches!(self.numeric(), 300..400)
    }

    #[must_use]
    pub const fn is_client_error(&self) -> bool {
        matches!(self.numeric(), 400..500)
    }

    #[must_use]
    pub const fn is_server_error(&self) -> bool {
        matches!(self.numeric(), 500..600)
    }

    #[must_use]
    pub const fn is_error(&self) -> bool {
        self.is_client_error() || self.is_server_error()
    }

    #[must_use]
    pub const fn allowed_to_have_body(&self) -> bool {
        !matches!(self.0, 100..200 | 204 | 304)
    }

    #[must_use]
    pub const fn textual_description(&self) -> Option<&'static str> {
        let description = match *self {
            Self::CONTINUE => "continue",
            Self::SWITCHING_PROTOCOLS => "switching protocols",
            Self::PROCESSING => "processing",
            Self::EARLY_HINTS => "early hints",
            Self::OK => "ok",
            Self::CREATED => "created",
            Self::ACCEPTED => "accepted",
            Self::NON_AUTHORITATIVE_INFORMATION => "non-authoritative information",
            Self::NO_CONTENT => "no content",
            Self::RESET_CONTENT => "reset content",
            Self::PARTIAL_CONTENT => "partial content",
            Self::MULTI_STATUS => "multi status",
            Self::ALREADY_REPORTED => "already reported",
            Self::IM_USED => "instance manipulation used",
            Self::MULTIPLE_CHOICES => "multiple choices",
            Self::MOVED_PERMANENTLY => "moved permanently",
            Self::FOUND => "found",
            Self::SEE_OTHER => "see other",
            Self::NOT_MODIFIED => "not modified",
            Self::USE_PROXY => "use proxy",
            Self::TEMPORARY_REDIRECT => "temporary redirect",
            Self::PERMANENT_REDIRECT => "permanent redirect",
            Self::BAD_REQUEST => "bad request",
            Self::UNAUTHORIZED => "unauthorized",
            Self::PAYMENT_REQUIRED => "payment required",
            Self::FORBIDDEN => "forbidden",
            Self::NOT_FOUND => "not found",
            Self::METHOD_NOT_ALLOWED => "method not allowed",
            Self::NOT_ACCEPTABLE => "not acceptable",
            Self::PROXY_AUTHENTICATION_REQUIRED => "proxy authentication required",
            Self::REQUEST_TIMEOUT => "request timeout",
            Self::CONFLICT => "conflict",
            Self::GONE => "gone",
            Self::LENGTH_REQUIRED => "length required",
            Self::PRECONDITION_FAILED => "precondition failed",
            Self::PAYLOAD_TOO_LARGE => "payload too large",
            Self::URI_TOO_LONG => "uri too long",
            Self::UNSUPPORTED_MEDIA_TYPE => "unsupported media type",
            Self::RANGE_NOT_SATISFIABLE => "range not satisfiable",
            Self::EXPECTATION_FAILED => "expectation failed",
            Self::IM_A_TEAPOT => "i am a teapot",
            Self::MISDIRECTED_REQUEST => "misdirected request",
            Self::UNPROCESSABLE_ENTITY => "unprocessable entity",
            Self::LOCKED => "locked",
            Self::FAILED_DEPENDENCY => "failed dependency",
            Self::TOO_EARLY => "too early",
            Self::UPGRADE_REQUIRED => "upgrade required",
            Self::PRECONDITION_REQUIRED => "precondition required",
            Self::TOO_MANY_REQUESTS => "too many requests",
            Self::REQUEST_HEADER_FIELD_TOO_LARGE => "request header field too large",
            Self::UNAVAILABLE_FOR_LEGAL_REASONS => "unavailable for legal reasons",
            Self::INTERNAL_SERVER_ERROR => "internal server error",
            Self::NOT_IMPLEMENTED => "not implemented",
            Self::BAD_GATEWAY => "bad gateway",
            Self::SERVICE_UNAVAILABLE => "service unavailable",
            Self::GATEWAY_TIMEOUT => "gateway timeout",
            Self::HTTP_VERSION_NOT_SUPPORTED => "http version not supported",
            Self::VARIANT_ALSO_NEGOTIATES => "variant also negotiates",
            Self::INSUFFICIENT_STORAGE => "insufficient storage",
            Self::LOOP_DETECTED => "loop detected",
            Self::NOT_EXTENDED => "not extended",
            Self::NETWORK_AUTHENTICATION_REQUIRED => "network authentication required",
            _ => return None,
        };

        Some(description)
    }
}

impl FromStr for StatusCode {
    type Err = <u32 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let status_code = Self(u16::from_str(s)?);
        Ok(status_code)
    }
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = self.textual_description() {
            write!(f, "{} ({description})", self.0)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl std::error::Error for StatusCode {}
