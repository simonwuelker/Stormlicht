/// Errors that can occur during URL parsing
///
/// [Specification](https://url.spec.whatwg.org/#validation-error)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValidationError {
    /// The input’s host contains a forbidden domain code point.
    ///
    /// ## Example
    /// Hosts are percent-decoded before being processed when the URL is special, which would result in the following host portion becoming "exa#mple.org" and thus triggering this error.
    ///
    /// `"https://exa%23mple.org"`
    DomainInvalidCodepoint,

    ///  An opaque host (in a URL that is not special) contains a forbidden host code point.
    ///
    /// ## Example
    /// `"foo://exa[mple.org" `
    HostInvalidCodepoint,

    /// An IPv4 address ends with a `.`
    ///
    /// ## Example
    /// `"https://127.0.0.1./"`
    IPv4EmptyPart,

    /// An IPv4 address does not consist of exactly 4 parts.
    ///
    /// ## Example
    /// `"https://1.2.3.4.5/"`
    IPv4TooManyParts,

    /// An IPv4 address part is not numeric.
    ///
    /// ## Example
    /// `"https://test.42"`
    IPv4NonNumericPart,

    /// The IPv4 address contains numbers expressed using hexadecimal or octal digits.
    ///
    /// ## Example
    /// `"https://127.0.0x0.1"`
    IPv4NonDecimalPart,

    /// An IPv4 address part exceeds 255.
    ///
    /// ## Example
    /// `"https://255.255.4000.1"`
    IPv4OutOfRangePart,

    /// An IPv6 address is missing the closing U+005D (]).
    ///
    /// ## Example
    /// `"https://[::1"`
    IPv6Unclosed,

    /// An IPv6 address begins with improper compression.
    ///
    /// ## Example
    /// `"https://[:1]"`
    IPv6InvalidCompression,

    /// An IPv6 address contains more than 8 pieces.
    ///
    /// ## Example
    /// `"https://[1:2:3:4:5:6:7:8:9]"`
    IPv6TooManyPieces,

    /// An IPv6 address is compressed in more than one spot.
    ///
    /// ## Example
    /// `"https://[1::1::1]"`
    IPv6MultipleCompression,

    /// An IPv6 address contains a code point that is neither an ASCII hex digit nor a U+003A (:).
    /// Or it unexpectedly ends.
    ///
    /// ## Examples
    /// * `"https://[1:2:3!:4]"`
    /// * `"https://[1:2:3:]" `
    IPv6InvalidCodepoint,

    /// An uncompressed IPv6 address contains fewer than 8 pieces.
    ///
    /// ## Example
    /// `"https://[1:2:3]"`
    IPv6TooFewPieces,

    /// An IPv6 address with IPv4 address syntax: the IPv6 address has more than 6 pieces.
    ///
    /// ## Example
    /// `"https://[1:1:1:1:1:1:1:127.0.0.1]" `
    IPv4InIPv6TooManyPieces,

    /// An IPv6 address with IPv4 address syntax:
    /// * An IPv4 part is empty or contains a non-ASCII digit.
    /// * An IPv4 part contains a leading 0.
    /// * There are too many IPv4 parts.
    ///
    /// ## Examples
    /// * `"https://[ffff::.0.0.1]"`
    /// * `"https://[ffff::127.0.xyz.1]"`
    /// * `"https://[ffff::127.0xyz]"`
    /// * `"https://[ffff::127.00.0.1]"`
    /// * `"https://[ffff::127.0.0.1.2]"`
    IPv4InIPv6InvalidCodepoint,

    /// An IPv6 address with IPv4 address syntax: an IPv4 part exceeds 255.
    ///
    /// ## Example
    /// `"https://[ffff::127.0.0.4000]"`
    IPv4InIPv6OutOfRangePart,

    /// An IPv6 address with IPv4 address syntax: an IPv4 address contains too few parts.
    ///
    /// ## Example
    /// `"https://[ffff::127.0.0]"`
    IPv4InIPv6TooFewParts,

    /// A code point is found that is not a URL unit.
    ///
    /// ## Examples
    /// * `"https://example.org/>"`
    /// * `" https://example.org "`
    /// * `"https://example.org/%s"`
    InvalidURLUnit,

    /// The input’s scheme is not followed by "//".
    ///
    /// ## Examples
    /// * `"file:c:/my-secret-folder"`
    /// * `"https:example.org"`
    SpecialSchemeMissingFollowingSolidus,

    /// The input is missing a scheme,
    /// because it does not begin with an ASCII alpha,
    /// and either no base URL was provided or the base URL
    /// cannot be used as a base URL because it has an opaque path.
    MissingSchemeNonRelativeURL,

    /// The URL has a special scheme and it uses `\` instead of `/`.
    ///
    /// ## Example
    /// `"https://example.org\path\to\file"`
    InvalidReverseSolidus,

    /// The input includes credentials.
    ///
    /// ## Examples
    /// * `"https://user@example.org"`
    /// * `"https://user:pass@"`
    InvalidCredentials,

    /// The input has a special scheme, but does not contain a host.
    ///
    /// ## Examples
    /// * `"https://#fragment"`
    /// * `"https://:443"`
    HostMissing,

    /// The input’s port is too big.
    ///
    /// ## Example
    /// `"https://example.org:70000"`
    PortOutOfRange,

    /// The input’s port is invalid.
    ///
    /// ## Example
    /// `"https://example.org:7z"`
    PortInvalid,

    /// The input is a relative-URL string that starts with a Windows drive letter and the base URL’s scheme is "file".
    FileInvalidWindowsDriveLetter,

    /// A file: URL’s host is a Windows drive letter.
    ///
    /// ## Example
    /// `"file://c:"`
    FileInvalidWindowsDriveLetterHost,
}

pub trait ValidationErrorHandler {
    fn validation_error(&mut self, validation_error: ValidationError);
}

#[derive(Clone, Copy)]
pub struct IgnoreValidationErrors;

impl ValidationErrorHandler for IgnoreValidationErrors {
    fn validation_error(&mut self, _validation_error: ValidationError) {}
}
