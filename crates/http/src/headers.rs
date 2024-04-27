//! <https://www.rfc-editor.org/rfc/rfc2616#section-4.2>

use sl_std::ascii;
use std::{collections::HashMap, fmt};

/// Thin wrapper around a [HashMap] to provide case-insensitive
/// key lookup, as is required for HTTP Headers.
#[derive(Clone, Debug, Default)]
pub struct Headers {
    internal: HashMap<Header, String>,
}

impl Headers {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            internal: HashMap::with_capacity(capacity),
        }
    }

    pub fn clear(&mut self) {
        self.internal.clear()
    }

    pub fn get(&self, header: Header) -> Option<&str> {
        self.internal.get(&header).map(String::as_str)
    }

    pub fn set(&mut self, header: Header, value: String) {
        self.internal.insert(header, value);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Header, &str)> {
        self.internal
            .iter()
            .map(|(key, value)| (key, value.as_str()))
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Header {
    /// A header name with a predefined meaning
    Defined(DefinedHeader),

    /// A header whose meaning we don't understand
    ///
    /// Maybe the application layer knows more.
    Custom(ascii::String),
}

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl From<DefinedHeader> for Header {
    fn from(value: DefinedHeader) -> Self {
        Self::Defined(value)
    }
}

macro_rules! header_byte_mapping {
    ($($ident: literal => $header: path,)*) => (
        pub fn from_lowercase_str(name: &ascii::Str) -> Self {
            match name.as_str() {
                $(
                    $ident => $header,
                )*
                _ => {
                    // We don't recognize the header, lets store it as-is and let the
                    // application layer deal with it.
                    Self::Custom(name.to_owned())
                },
            }
        }

        pub fn as_str(&self) -> &str {
            match self {
                $(
                    &$header => $ident,
                )*
                Self::Custom(name) => name.as_str(),
            }
        }
    )
}

impl Header {
    /// Defines the authentication method that should be used to access a resource.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/WWW-Authenticate)
    pub const WWW_AUTHENTICATE: Self = Self::Defined(DefinedHeader::WwwAuthenticate);

    /// Contains the credentials to authenticate a user-agent with a server.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Authorization)
    pub const AUTHORIZATION: Self = Self::Defined(DefinedHeader::Authorization);

    /// Defines the authentication method that should be used to access a
    /// resource behind a proxy server.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Proxy-Authenticate)
    pub const PROXY_AUTHENTICATE: Self = Self::Defined(DefinedHeader::ProxyAuthenticate);

    /// Contains the credentials to authenticate a user agent with a proxy server.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Proxy-Authorization)
    pub const PROXY_AUTHORIZATION: Self = Self::Defined(DefinedHeader::ProxyAuthorization);

    /// The time, in seconds, that the object has been in a proxy cache.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Age)
    pub const AGE: Self = Self::Defined(DefinedHeader::Age);

    /// Directives for caching mechanisms in both requests and responses.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control)
    pub const CACHE_CONTROL: Self = Self::Defined(DefinedHeader::CacheControl);

    /// Clears browsing data (e.g. cookies, storage, cache) associated with the requesting website.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Clear-Site-Data)
    pub const CLEAR_SITE_DATA: Self = Self::Defined(DefinedHeader::ClearSiteData);

    /// The date/time after which the response is considered stale.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Expires)
    pub const EXPIRES: Self = Self::Defined(DefinedHeader::Expires);

    /// Specifies a set of rules that define how a URL's query parameters will affect
    /// cache matching. These rules dictate whether the same URL with different
    /// URL parameters should be saved as separate browser cache entries.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/No-Vary-Search)
    pub const NO_VARY_SEARCH: Self = Self::Defined(DefinedHeader::NoVarySearch);

    /// The last modification date of the resource, used to compare several versions of the
    /// same resource. It is less accurate than ETag, but easier to calculate in some
    /// environments. Conditional requests using `If-Modified-Since` and `If-Unmodified-Since` use
    /// this value to change the behavior of the request.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Last-Modified)
    pub const LAST_MODIFIED: Self = Self::Defined(DefinedHeader::LastModified);

    /// A unique string identifying the version of the resource. Conditional requests using
    /// `If-Match` and `If-None-Match` use this value to change the behavior of the request.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/ETag)
    pub const ETAG: Self = Self::Defined(DefinedHeader::ETag);

    /// Makes the request conditional, and applies the method only if the stored resource
    /// matches one of the given ETags.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/If-Match)
    pub const IF_MATCH: Self = Self::Defined(DefinedHeader::IfMatch);

    /// Makes the request conditional, and applies the method only if the stored resource
    /// doesn't match any of the given ETags. This is used to update caches (for safe requests),
    /// or to prevent uploading a new resource when one already exists.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/If-None-Match)
    pub const IF_NONE_MATCH: Self = Self::Defined(DefinedHeader::IfNoneMatch);

    /// Makes the request conditional, and expects the resource to be transmitted only if
    /// it has been modified after the given date. This is used to transmit data only when
    /// the cache is out of date.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/If-Modified-Since)
    pub const IF_MODIFIED_SINCE: Self = Self::Defined(DefinedHeader::IfModifiedSince);

    /// Makes the request conditional, and expects the resource to be transmitted only if
    /// it has not been modified after the given date. This ensures the coherence of a
    /// new fragment of a specific range with previous ones, or to implement an optimistic
    /// concurrency control system when modifying existing documents.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/If-Unmodified-Since)
    pub const IF_UNMODIFIED_SINCE: Self = Self::Defined(DefinedHeader::IfUnmodifiedSince);

    /// Determines how to match request headers to decide whether a cached response can be
    /// used rather than requesting a fresh one from the origin server.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Vary)
    pub const VARY: Self = Self::Defined(DefinedHeader::Vary);

    /// Controls whether the network connection stays open after the current transaction finishes.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection)
    pub const CONNECTION: Self = Self::Defined(DefinedHeader::Connection);

    /// Controls how long a persistent connection should stay open.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Keep-Alive)
    pub const KEEP_ALIVE: Self = Self::Defined(DefinedHeader::KeepAlive);

    /// Informs the server about the types of data that can be sent back.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept)
    pub const ACCEPT: Self = Self::Defined(DefinedHeader::Accept);

    /// The encoding algorithm, usually a compression algorithm, that can be
    /// used on the resource sent back.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Encoding)
    pub const ACCEPT_ENCODING: Self = Self::Defined(DefinedHeader::AcceptEncoding);

    /// Informs the server about the human language the server is expected to send back.
    /// This is a hint and is not necessarily under the full control of the user:
    /// the server should always pay attention not to override an explicit
    /// user choice (like selecting a language from a dropdown).
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Language)
    pub const ACCEPT_LANGUAGE: Self = Self::Defined(DefinedHeader::AcceptLanguage);

    /// Indicates expectations that need to be fulfilled by the server to properly handle the request.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Expect)
    pub const EXPECT: Self = Self::Defined(DefinedHeader::Expect);

    /// When using `TRACE`, indicates the maximum number of hops the request can do
    /// before being reflected to the sender.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Max-Forwards)
    pub const MAX_FORWARDS: Self = Self::Defined(DefinedHeader::MaxForwards);

    /// Contains stored HTTP cookies previously sent by the server with the `Set-Cookie` header.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cookie)
    pub const COOKIE: Self = Self::Defined(DefinedHeader::Cookie);

    /// Send cookies from the server to the user-agent.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie)
    pub const SET_COOKIE: Self = Self::Defined(DefinedHeader::SetCookie);

    /// Indicates whether the response to the request can be exposed when the
    /// credentials flag is true.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Credentials)
    pub const ACCESS_CONTROL_ALLOW_CREDENTIALS: Self =
        Self::Defined(DefinedHeader::AccessControlAllowCredentials);

    /// Used in response to a preflight request to indicate which HTTP headers can be
    /// used when making the actual request.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Headers)
    pub const ACCESS_CONTROL_ALLOW_HEADERS: Self =
        Self::Defined(DefinedHeader::AccessControlAllowHeaders);

    /// Specifies the methods allowed when accessing the resource in response to a preflight request.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Methods)
    pub const ACCESS_CONTROL_ALLOW_METHODS: Self =
        Self::Defined(DefinedHeader::AccessControlAllowMethods);

    /// Indicates whether the response can be shared.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Origin)
    pub const ACCESS_CONTROL_ALLOW_ORIGIN: Self =
        Self::Defined(DefinedHeader::AccessControlAllowOrigin);

    /// Indicates which headers can be exposed as part of the response by listing their names.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Expose-Headers)
    pub const ACCESS_CONTROL_EXPOSE_HEADERS: Self =
        Self::Defined(DefinedHeader::AccessControlExposeHeaders);

    /// Indicates how long the results of a preflight request can be cached.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Max-Age)
    pub const ACCESS_CONTROL_MAX_AGE: Self = Self::Defined(DefinedHeader::AccessControlMaxAge);

    /// Used when issuing a preflight request to let the server know which HTTP headers will
    /// be used when the actual request is made.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Request-Headers)
    pub const ACCESS_CONTROL_REQUEST_HEADERS: Self =
        Self::Defined(DefinedHeader::AccessControlRequestHeaders);

    /// Used when issuing a preflight request to let the server know which HTTP method will be
    /// used when the actual request is made.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Request-Method)
    pub const ACCESS_CONTROL_REQUEST_METHOD: Self =
        Self::Defined(DefinedHeader::AccessControlRequestMethod);

    /// Indicates where a fetch originates from.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Origin)
    pub const ORIGIN: Self = Self::Defined(DefinedHeader::Origin);

    /// Specifies origins that are allowed to see values of attributes retrieved via features
    /// of the Resource Timing API, which would otherwise be reported as zero due to
    /// cross-origin restrictions.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Timing-Allow-Origin)
    pub const TIMING_ALLOW_ORIGIN: Self = Self::Defined(DefinedHeader::TimingAllowOrigin);

    /// Indicates if the resource transmitted should be displayed inline
    /// (default behavior without the header), or if it should be handled like a download
    /// and the browser should present a "Save As" dialog.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Disposition)
    pub const CONTENT_DISPOSITION: Self = Self::Defined(DefinedHeader::ContentDisposition);

    /// The size of the resource, in decimal number of bytes.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Length)
    pub const CONTENT_LENGTH: Self = Self::Defined(DefinedHeader::ContentLength);

    /// Indicates the media type of the resource.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Type)
    pub const CONTENT_TYPE: Self = Self::Defined(DefinedHeader::ContentType);

    /// Used to specify the compression algorithm.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Encoding)
    pub const CONTENT_ENCODING: Self = Self::Defined(DefinedHeader::ContentEncoding);

    /// Describes the human language(s) intended for the audience, so that it allows
    /// a user to differentiate according to the users' own preferred language.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Language)
    pub const CONTENT_LANGUAGE: Self = Self::Defined(DefinedHeader::ContentLanguage);

    /// Indicates an alternate location for the returned data.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Location)
    pub const CONTENT_LOCATION: Self = Self::Defined(DefinedHeader::ContentLocation);

    /// Contains information from the client-facing side of proxy servers that is altered
    /// or lost when a proxy is involved in the path of the request.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Forwarded)
    pub const FORWARDED: Self = Self::Defined(DefinedHeader::Forwarded);

    /// Added by proxies, both forward and reverse proxies, and can appear in the request
    /// headers and the response headers.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Via)
    pub const VIA: Self = Self::Defined(DefinedHeader::Via);

    /// Indicates the URL to redirect a page to.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Location)
    pub const LOCATION: Self = Self::Defined(DefinedHeader::Location);

    /// Directs the browser to reload the page or redirect to another.

    /// Takes the same value as the `meta` element with `http-equiv="refresh"`
    pub const REFRESH: Self = Self::Defined(DefinedHeader::Refresh);

    /// Contains an Internet email address for a human user who controls the requesting user agent.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/From)
    pub const FROM: Self = Self::Defined(DefinedHeader::From);

    /// Specifies the domain name of the server (for virtual hosting), and (optionally)
    /// the TCP port number on which the server is listening.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Host)
    pub const HOST: Self = Self::Defined(DefinedHeader::Host);

    /// The address of the previous web page from which a link to the
    /// currently requested page was followed.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Referer)
    pub const REFERER: Self = Self::Defined(DefinedHeader::Referer);

    /// Governs which referrer information sent in the Referer header should
    /// be included with requests made.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Referrer-Policy)
    pub const REFERRER_POLICY: Self = Self::Defined(DefinedHeader::ReferrerPolicy);

    /// Contains a characteristic string that allows the network protocol peers to identify
    /// the application type, operating system, software vendor or software version of the
    /// requesting software user agent.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/User-Agent)
    pub const USER_AGENT: Self = Self::Defined(DefinedHeader::UserAgent);

    /// Lists the set of HTTP request methods supported by a resource.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Allow)
    pub const ALLOW: Self = Self::Defined(DefinedHeader::Allow);

    /// Contains information about the software used by the origin server to handle the request.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Server)
    pub const SERVER: Self = Self::Defined(DefinedHeader::Server);

    /// Indicates if the server supports range requests, and if so in which
    /// unit the range can be expressed.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Ranges)
    pub const ACCEPT_RANGES: Self = Self::Defined(DefinedHeader::AcceptRanges);

    /// Indicates the part of a document that the server should return.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Range)
    pub const RANGE: Self = Self::Defined(DefinedHeader::Range);

    /// Creates a conditional range request that is only fulfilled if the given
    /// etag or date matches the remote resource. Used to prevent downloading
    /// two ranges from incompatible version of the resource.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/If-Range)
    pub const IF_RANGE: Self = Self::Defined(DefinedHeader::IfRange);

    /// Indicates where in a full body message a partial message belongs.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Range)
    pub const CONTENT_RANGE: Self = Self::Defined(DefinedHeader::ContentRange);

    /// Allows a server to declare an embedder policy for a given document.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cross-Origin-Embedder-Policy)
    pub const CROSS_ORIGIN_EMBEDDER_POLICY: Self =
        Self::Defined(DefinedHeader::CrossOriginEmbedderPolicy);

    /// Prevents other domains from opening/controlling a window.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cross-Origin-Opener-Policy)
    pub const CROSS_ORIGIN_OPENER_POLICY: Self =
        Self::Defined(DefinedHeader::CrossOriginOpenerPolicy);

    /// Prevents other domains from reading the response of the resources to which this header is applied
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cross-Origin-Resource-Policy)
    pub const CROSS_ORIGIN_RESOURCE_POLICY: Self =
        Self::Defined(DefinedHeader::CrossOriginResourcePolicy);

    /// Controls resources the user agent is allowed to load for a given page.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Security-Policy)
    pub const CONTENT_SECURITY_POLICY: Self = Self::Defined(DefinedHeader::ContentSecurityPolicy);

    /// Allows web developers to experiment with policies by monitoring, but not enforcing,
    /// their effects. These violation reports consist of JSON documents sent via an
    /// HTTP POST request to the specified URI.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Security-Policy-Report-Only)
    pub const CONTENT_SECURITY_POLICY_REPORT_ONLY: Self =
        Self::Defined(DefinedHeader::ContentSecurityPolicyReportOnly);

    /// Provides a mechanism to allow and deny the use of browser features in a
    /// website's own frame, and in `<iframe>`s that it embeds.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Permissions-Policy)
    pub const PERMISSIONS_POLICY: Self = Self::Defined(DefinedHeader::PermissionsPolicy);

    /// Force communication using HTTPS instead of HTTP.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Strict-Transport-Security)
    pub const STRICT_TRANSPORT_SECURITY: Self =
        Self::Defined(DefinedHeader::StrictTransportSecurity);

    /// Sends a signal to the server expressing the client's preference for an encrypted and
    /// authenticated response, and that it can successfully handle the
    /// `upgrade-insecure-requests` directive.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Upgrade-Insecure-Requests)
    pub const UPGRADE_INSECURE_REQUESTS: Self =
        Self::Defined(DefinedHeader::UpgradeInsecureRequests);

    /// Disables MIME sniffing and forces browser to use the type given in Content-Type.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Content-Type-Options)
    pub const CONTENT_TYPE_OPTIONS: Self = Self::Defined(DefinedHeader::ContentTypeOptions);

    /// Indicates whether a browser should be allowed to render a page in a
    /// `<frame>`, `<iframe>`, `<embed>` or `<object>`.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Frame-Options)
    pub const FRAME_OPTIONS: Self = Self::Defined(DefinedHeader::FrameOptions);

    /// Specifies if a cross-domain policy file (crossdomain.xml) is allowed.
    /// The file may define a policy to grant clients, such as Adobe's Flash Player
    /// (now obsolete), Adobe Acrobat, Microsoft Silverlight (now obsolete), or
    /// Apache Flex, permission to handle data across domains that would otherwise
    /// be restricted due to the Same-Origin Policy.
    pub const PERMITTED_CROSS_DOMAIN_POLICIES: Self =
        Self::Defined(DefinedHeader::PermittedCrossDomainPolicies);

    /// May be set by hosting environments or other frameworks and contains information
    /// about them while not providing any usefulness to the application or its visitors.
    /// Unset this header to avoid exposing potential vulnerabilities.
    pub const POWERED_BY: Self = Self::Defined(DefinedHeader::PoweredBy);

    /// Enables cross-site scripting filtering.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-XSS-Protection)
    pub const XSS_PROTECTION: Self = Self::Defined(DefinedHeader::XssProtection);

    /// Indicates the relationship between a request initiator's origin and its target's origin.
    ///
    /// It is a Structured Header whose value is a token with possible values
    /// `cross-site`, `same-origin`, `same-site`, and `none`.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Sec-Fetch-Site)
    pub const SEC_FETCH_SITE: Self = Self::Defined(DefinedHeader::SecFetchSite);

    /// Indicates the request's mode to a server.
    ///
    /// It is a Structured Header whose value is a token with possible values
    /// `cors`, `navigate`, `no-cors`, `same-origin`, and `websocket`.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Sec-Fetch-Mode)
    pub const SEC_FETCH_MODE: Self = Self::Defined(DefinedHeader::SecFetchMode);

    /// Indicates whether or not a navigation request was triggered by user activation.
    ///
    /// It is a Structured Header whose value is a boolean so possible values are
    /// `?0` for false and `?1` for true.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Sec-Fetch-User)
    pub const SEC_FETCH_USER: Self = Self::Defined(DefinedHeader::SecFetchUser);

    /// Indicates the request's destination.
    ///
    /// It is a Structured Header whose value is a token with possible values
    /// `audio`, `audioworklet`, `document`, `embed`, `empty`, `font`, `image`, `manifest`, `object`,
    /// `paintworklet`, `report`, `script`, `serviceworker`, `sharedworker`, `style`, `track`, `video`,
    /// `worker`, and `xslt`.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Sec-Fetch-Dest)
    pub const SEC_FETCH_DEST: Self = Self::Defined(DefinedHeader::SecFetchDest);

    /// Indicates the purpose of the request, when the purpose is something other
    /// than immediate use by the user-agent. The header currently has one possible
    /// value, `prefetch`, which indicates that the resource is being fetched preemptively
    /// for a possible future navigation.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Sec-Purpose)
    pub const SEC_PURPOSE: Self = Self::Defined(DefinedHeader::SecPurpose);

    /// A request header sent in preemptive request to fetch() a resource during service
    /// worker boot. The value, which is set with `NavigationPreloadManager.setHeaderValue()`,
    /// can be used to inform a server that a different resource should be returned than in a
    /// normal `fetch()` operation.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Service-Worker-Navigation-Preload)
    pub const SERVICE_WORKER_NAVIGATION_PRELOAD: Self =
        Self::Defined(DefinedHeader::ServiceWorkerNavigationPreload);

    /// Used to specify a server endpoint for the browser to send warning and error reports to.
    pub const REPORT_TO: Self = Self::Defined(DefinedHeader::ReportTo);

    /// Specifies the form of encoding used to safely transfer the resource to the user.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Transfer-Encoding)
    pub const TRANSFER_ENCODING: Self = Self::Defined(DefinedHeader::TransferEncoding);

    /// Specifies the transfer encodings the user agent is willing to accept.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/TE)
    pub const TE: Self = Self::Defined(DefinedHeader::Te);

    /// Allows the sender to include additional fields at the end of chunked message.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Trailer)
    pub const TRAILER: Self = Self::Defined(DefinedHeader::Trailer);

    /// Used to list alternate ways to reach this service.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Alt-Svc)
    pub const ALT_SVC: Self = Self::Defined(DefinedHeader::AltSvc);

    /// Used to identify the alternative service in use.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Alt-Used)
    pub const ALT_USED: Self = Self::Defined(DefinedHeader::AltUsed);

    /// Contains the date and time at which the message was originated.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Date)
    pub const DATE: Self = Self::Defined(DefinedHeader::Date);

    /// This entity-header field provides a means for serializing one or more links in HTTP headers.
    /// It is semantically equivalent to the HTML `<link>` element.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Link)
    pub const LINK: Self = Self::Defined(DefinedHeader::Link);

    /// Indicates how long the user agent should wait before making a follow-up request.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Retry-After)
    pub const RETRY_AFTER: Self = Self::Defined(DefinedHeader::RetryAfter);

    /// Communicates one or more metrics and descriptions for the given request-response cycle.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Server-Timing)
    pub const SERVER_TIMING: Self = Self::Defined(DefinedHeader::ServerTiming);

    /// Used to remove the path restriction by including this header in the response
    /// of the Service Worker script.
    pub const SERVICE_WORKER_ALLOWED: Self = Self::Defined(DefinedHeader::ServiceWorkerAllowed);

    /// Links generated code to a source map.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/SourceMap)
    pub const SOURCE_MAP: Self = Self::Defined(DefinedHeader::SourceMap);

    /// This HTTP/1.1 (only) header can be used to upgrade an already established
    /// client/server connection to a different protocol (over the same transport protocol).
    /// For example, it can be used by a client to upgrade a connection from HTTP 1.1 to
    /// HTTP 2.0, or an HTTP or HTTPS connection into a WebSocket.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Upgrade)
    pub const UPGRADE: Self = Self::Defined(DefinedHeader::Upgrade);

    /// Servers can advertise support for Client Hints using the `Accept-CH` header field or an
    /// equivalent HTML `<meta>` element with http-equiv attribute.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-CH)
    pub const ACCEPT_CH: Self = Self::Defined(DefinedHeader::AcceptClientHints);

    /// Servers use `Critical-CH` along with `Accept-CH` to specify that accepted client hints
    /// are also critical client hints.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Critical-CH)
    pub const CRITICAL_CH: Self = Self::Defined(DefinedHeader::CriticalClientHints);

    /// User agent's branding and version.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Sec-CH-UA)
    pub const SEC_CH_UA: Self = Self::Defined(DefinedHeader::SecClientHintUserAgent);

    /// User agent's underlying platform architecture.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Sec-CH-UA-Arch)
    pub const SEC_CH_UA_ARCH: Self =
        Self::Defined(DefinedHeader::SecClientHintUserAgentArchitecture);

    /// User agent's underlying CPU architecture bitness (for example "64" bit).
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Sec-CH-UA-Bitness)
    pub const SEC_CH_UA_BITNESS: Self = Self::Defined(DefinedHeader::SecClientHintUserAgentBitness);

    /// Full version for each brand in the user agent's brand list.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Sec-CH-UA-Full-Version-List)
    pub const SEC_CH_UA_FULL_VERSION_LIST: Self =
        Self::Defined(DefinedHeader::SecClientHintUserAgentFullVersionList);

    /// User agent is running on a mobile device or, more generally, prefers a "mobile" user experience.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Sec-CH-UA-Mobile)
    pub const SEC_CH_UA_MOBILE: Self = Self::Defined(DefinedHeader::SecClientHintUserAgentMobile);

    /// User agent's device model.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Sec-CH-UA-Model)
    pub const SEC_CH_UA_MODEL: Self = Self::Defined(DefinedHeader::SecClientHintUserAgentModel);

    /// User agent's underlying operation system/platform.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Sec-CH-UA-Platform)
    pub const SEC_CH_UA_PLATFORM: Self =
        Self::Defined(DefinedHeader::SecClientHintUserAgentPlatform);

    /// User agent's underlying operation system version.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Sec-CH-UA-Platform-Version)
    pub const SEC_CH_UA_PLATFORM_VERSION: Self =
        Self::Defined(DefinedHeader::SecClientHintUserAgentPlatformVersion);

    /// User's preference of dark or light color scheme.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Sec-CH-UA-Prefers-Color-Scheme)
    pub const SEC_CH_UA_PREFERS_COLOR_SCHEME: Self =
        Self::Defined(DefinedHeader::SecClientHintUserAgentPrefersColorScheme);

    /// User's preference to see fewer animations and content layout shifts.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Sec-CH-UA-Prefers-Reduced-Motion)
    pub const SEC_CH_UA_PREFERS_REDUCED_MOTION: Self =
        Self::Defined(DefinedHeader::SecClientHintUserAgentPrefersReducedMotion);

    /// Approximate amount of available client RAM memory. This is part of the Device Memory API.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Device-Memory)
    pub const DEVICE_MEMORY: Self = Self::Defined(DefinedHeader::DeviceMemory);

    /// Identifies the originating IP addresses of a client connecting to a web server
    /// through an HTTP proxy or a load balancer.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Forwarded-For)
    pub const X_FORWARDED_FOR: Self =
        Self::Defined(DefinedHeader::NonStandard(NonStandardHeader::ForwardedFor));

    /// Identifies the original host requested that a client used to connect to your
    /// proxy or load balancer.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Forwarded-Host)
    pub const X_FORWARDED_HOST: Self =
        Self::Defined(DefinedHeader::NonStandard(NonStandardHeader::ForwardedHost));

    /// Identifies the protocol (HTTP or HTTPS) that a client used to connect to your
    /// proxy or load balancer.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Forwarded-Proto)
    pub const X_FORWARDED_PROTO: Self = Self::Defined(DefinedHeader::NonStandard(
        NonStandardHeader::ForwardedProto,
    ));

    /// Controls DNS prefetching, a feature by which browsers proactively perform domain name
    /// resolution on both links that the user may choose to follow as well as URLs for items
    /// referenced by the document, including images, CSS, JavaScript, and so forth.
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-DNS-Prefetch-Control)
    pub const X_DNS_PREFETCH_CONTROL: Self = Self::Defined(DefinedHeader::NonStandard(
        NonStandardHeader::DnsPrefetchControl,
    ));

    header_byte_mapping!(
        "www-authenticate" => Self::WWW_AUTHENTICATE,
        "authorization" => Self::AUTHORIZATION,
        "proxy-authenticate" => Self::PROXY_AUTHENTICATE,
        "proxy-authorization" => Self::PROXY_AUTHORIZATION,
        "age" => Self::AGE,
        "cache-control" => Self::CACHE_CONTROL,
        "clear-site-data" => Self::CLEAR_SITE_DATA,
        "expires" => Self::EXPIRES,
        "no-vary-search" => Self::NO_VARY_SEARCH,
        "last-modified" => Self::LAST_MODIFIED,
        "etag" => Self::ETAG,
        "if-match" => Self::IF_MATCH,
        "if-none-match" => Self::IF_NONE_MATCH,
        "if-modified-since" => Self::IF_MODIFIED_SINCE,
        "if-unmodified-since" => Self::IF_UNMODIFIED_SINCE,
        "vary" => Self::VARY,
        "connection" => Self::CONNECTION,
        "keep-alive" => Self::KEEP_ALIVE,
        "accept" => Self::ACCEPT,
        "accept-encoding" => Self::ACCEPT_ENCODING,
        "accept-language" => Self::ACCEPT_LANGUAGE,
        "expect" => Self::EXPECT,
        "max-forwards" => Self::MAX_FORWARDS,
        "cookie" => Self::COOKIE,
        "set-cookie" => Self::SET_COOKIE,
        "access-control-allow-credentials" => Self::ACCESS_CONTROL_ALLOW_CREDENTIALS,
        "access-control-allow-headers" => Self::ACCESS_CONTROL_ALLOW_HEADERS,
        "access-control-allow-methods" => Self::ACCESS_CONTROL_ALLOW_METHODS,
        "access-control-allow-origin" => Self::ACCESS_CONTROL_ALLOW_ORIGIN,
        "access-control-expose-headers" => Self::ACCESS_CONTROL_EXPOSE_HEADERS,
        "access-control-max-age" => Self::ACCESS_CONTROL_MAX_AGE,
        "access-control-request-headers" => Self::ACCESS_CONTROL_REQUEST_HEADERS,
        "access-control-request-method" => Self::ACCESS_CONTROL_REQUEST_METHOD,
        "origin" => Self::ORIGIN,
        "timing-allow-origin" => Self::TIMING_ALLOW_ORIGIN,
        "content-disposition" => Self::CONTENT_DISPOSITION,
        "content-length" => Self::CONTENT_LENGTH,
        "content-type" => Self::CONTENT_TYPE,
        "content-encoding" => Self::CONTENT_ENCODING,
        "content-language" => Self::CONTENT_LANGUAGE,
        "content-location" => Self::CONTENT_LOCATION,
        "forwarded" => Self::FORWARDED,
        "via" => Self::VIA,
        "location" => Self::LOCATION,
        "refresh" => Self::REFRESH,
        "from" => Self::FROM,
        "host" => Self::HOST,
        "referer" => Self::REFERER,
        "referrer-policy" => Self::REFERRER_POLICY,
        "user-agent" => Self::USER_AGENT,
        "allow" => Self::ALLOW,
        "server" => Self::SERVER,
        "accept-ranges" => Self::ACCEPT_RANGES,
        "range" => Self::RANGE,
        "if-range" => Self::IF_RANGE,
        "content-range" => Self::CONTENT_RANGE,
        "cross-origin-embedder-policy" => Self::CROSS_ORIGIN_EMBEDDER_POLICY,
        "cross-origin-opener-policy" => Self::CROSS_ORIGIN_OPENER_POLICY,
        "cross-origin-resource-policy" => Self::CROSS_ORIGIN_RESOURCE_POLICY,
        "content-security-policy" => Self::CONTENT_SECURITY_POLICY,
        "content-security-policy-report-only" => Self::CONTENT_SECURITY_POLICY_REPORT_ONLY,
        "permissions-policy" => Self::PERMISSIONS_POLICY,
        "strict-transport-security" => Self::STRICT_TRANSPORT_SECURITY,
        "upgrade-insecure-requests" => Self::UPGRADE_INSECURE_REQUESTS,
        "x-content-type-options" => Self::CONTENT_TYPE_OPTIONS,
        "x-frame-options" => Self::FRAME_OPTIONS,
        "x-permitted-cross-domain-policies" => Self::PERMITTED_CROSS_DOMAIN_POLICIES,
        "x-powered-by" => Self::POWERED_BY,
        "x-xss-protection" => Self::XSS_PROTECTION,
        "sec-fetch-site" => Self::SEC_FETCH_SITE,
        "sec-fetch-mode" => Self::SEC_FETCH_MODE,
        "sec-fetch-user" => Self::SEC_FETCH_USER,
        "sec-fetch-dest" => Self::SEC_FETCH_DEST,
        "sec-purpose" => Self::SEC_PURPOSE,
        "service-worker-navigation-preload" => Self::SERVICE_WORKER_NAVIGATION_PRELOAD,
        "report-to" => Self::REPORT_TO,
        "transfer-encoding" => Self::TRANSFER_ENCODING,
        "te" => Self::TE,
        "trailer" => Self::TRAILER,
        "alt-svc" => Self::ALT_SVC,
        "alt-used" => Self::ALT_USED,
        "date" => Self::DATE,
        "link" => Self::LINK,
        "retry-after" => Self::RETRY_AFTER,
        "server-timing" => Self::SERVER_TIMING,
        "service-worker-allowed" => Self::SERVICE_WORKER_ALLOWED,
        "sourcemap" => Self::SOURCE_MAP,
        "upgrade" => Self::UPGRADE,
        "accept-ch" => Self::ACCEPT_CH,
        "critical-ch" => Self::CRITICAL_CH,
        "sec-ch-ua" => Self::SEC_CH_UA,
        "sec-ch-ua-arch" => Self::SEC_CH_UA_ARCH,
        "sec-ch-ua-bitness" => Self::SEC_CH_UA_BITNESS,
        "sec-ch-ua-full-version-list" => Self::SEC_CH_UA_FULL_VERSION_LIST,
        "sec-ch-ua-mobile" => Self::SEC_CH_UA_MOBILE,
        "sec-ch-ua-model" => Self::SEC_CH_UA_MODEL,
        "sec-ch-ua-platform" => Self::SEC_CH_UA_PLATFORM,
        "sec-ch-ua-platform-version" => Self::SEC_CH_UA_PLATFORM_VERSION,
        "sec-ch-ua-prefers-color-scheme" => Self::SEC_CH_UA_PREFERS_COLOR_SCHEME,
        "sec-ch-ua-prefers-reduced-motion" => Self::SEC_CH_UA_PREFERS_REDUCED_MOTION,
        "device-memory" => Self::DEVICE_MEMORY,
        "x-forwarded-for" => Self::X_FORWARDED_FOR,
        "x-forwarded-host" => Self::X_FORWARDED_HOST,
        "x-forwarded-proto" => Self::X_FORWARDED_PROTO,
        "x-dns-prefetch-control" => Self::X_DNS_PREFETCH_CONTROL,
    );
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DefinedHeader {
    /// Refer to [Header::WWW_AUTHENTICATE] for documentation
    WwwAuthenticate,

    /// Refer to [Header::AUTHORIZATION] for documentation
    Authorization,

    /// Refer to [Header::PROXY_AUTHENTICATE] for documentation
    ProxyAuthenticate,

    /// Refer to [Header::PROXY_AUTHORIZATION] for documentation
    ProxyAuthorization,

    /// Refer to [Header::AGE] for documentation
    Age,

    /// Refer to [Header::CACHE_CONTROL] for documentation
    CacheControl,

    /// Refer to [Header::CLEAR_SITE_DATA] for documentation
    ClearSiteData,

    /// Refer to [Header::EXPIRES] for documentation
    Expires,

    /// Refer to [Header::NO_VARY_SEARCH] for documentation
    NoVarySearch,

    /// Refer to [Header::LAST_MODIFIED] for documentation
    LastModified,

    /// Refer to [Header::ETAG] for documentation
    ETag,

    /// Refer to [Header::IF_MATCH] for documentation
    IfMatch,

    /// Refer to [Header::IF_NONE_MATCH] for documentation
    IfNoneMatch,

    /// Refer to [Header::IF_MODIFIED_SINCE] for documentation
    IfModifiedSince,

    /// Refer to [Header::IF_UNMODIFIED_SINCE] for documentation
    IfUnmodifiedSince,

    /// Refer to [Header::VARY] for documentation
    Vary,

    /// Refer to [Header::CONNECTION] for documentation
    Connection,

    /// Refer to [Header::KEEP_ALIVE] for documentation
    KeepAlive,

    /// Refer to [Header::ACCEPT] for documentation
    Accept,

    /// Refer to [Header::ACCEPT_ENCODING] for documentation
    AcceptEncoding,

    /// Refer to [Header::ACCEPT_LANGUAGE] for documentation
    AcceptLanguage,

    /// Refer to [Header::EXPECT] for documentation
    Expect,

    /// Refer to [Header::MAX_FORWARDS] for documentation
    MaxForwards,

    /// Refer to [Header::COOKIE] for documentation
    Cookie,

    /// Refer to [Header::SET_COOKIE] for documentation
    SetCookie,

    /// Refer to [Header::ACCESS_CONTROL_ALLOW_CREDENTIALS] for documentation
    AccessControlAllowCredentials,

    /// Refer to [Header::ACCESS_CONTROL_ALLOW_HEADERS] for documentation
    AccessControlAllowHeaders,

    /// Refer to [Header::ACCESS_CONTROL_ALLOW_METHODS] for documentation
    AccessControlAllowMethods,

    /// Refer to [Header::ACCESS_CONTROL_ALLOW_ORIGIN] for documentation
    AccessControlAllowOrigin,

    /// Refer to [Header::ACCESS_CONTROL_EXPOSE_HEADERS] for documentation
    AccessControlExposeHeaders,

    /// Refer to [Header::ACCESS_CONTROL_MAX_AGE] for documentation
    AccessControlMaxAge,

    /// Refer to [Header::ACCESS_CONTROL_REQUEST_HEADERS] for documentation
    AccessControlRequestHeaders,

    /// Refer to [Header::ACCESS_CONTROL_REQUEST_METHOD] for documentation
    AccessControlRequestMethod,

    /// Refer to [Header::ORIGIN] for documentation
    Origin,

    /// Refer to [Header::TIMING_ALLOW_ORIGIN] for documentation
    TimingAllowOrigin,

    /// Refer to [Header::CONTENT_DISPOSITION] for documentation
    ContentDisposition,

    /// Refer to [Header::CONTENT_LENGTH] for documentation
    ContentLength,

    /// Refer to [Header::CONTENT_TYPE] for documentation
    ContentType,

    /// Refer to [Header::CONTENT_ENCODING] for documentation
    ContentEncoding,

    /// Refer to [Header::CONTENT_LANGUAGE] for documentation
    ContentLanguage,

    /// Refer to [Header::CONTENT_LOCATION] for documentation
    ContentLocation,

    /// Refer to [Header::FORWARDED] for documentation
    Forwarded,

    /// Refer to [Header::VIA] for documentation
    Via,

    /// Refer to [Header::LOCATION] for documentation
    Location,

    /// Refer to [Header::REFRESH] for documentation.
    Refresh,

    /// Refer to [Header::FROM] for documentation
    From,

    /// Refer to [Header::HOST] for documentation
    Host,

    /// Refer to [Header::REFERER] for documentation
    Referer,

    /// Refer to [Header::REFERRER_POLICY] for documentation
    ReferrerPolicy,

    /// Refer to [Header::USER_AGENT] for documentation
    UserAgent,

    /// Refer to [Header::ALLOW] for documentation
    Allow,

    /// Refer to [Header::SERVER] for documentation
    Server,

    /// Refer to [Header::ACCEPT_RANGES] for documentation
    AcceptRanges,

    /// Refer to [Header::RANGE] for documentation
    Range,

    /// Refer to [Header::IF_RANGE] for documentation
    IfRange,

    /// Refer to [Header::CONTENT_RANGE] for documentation
    ContentRange,

    /// Refer to [Header::CROSS_ORIGIN_EMBEDDER_POLICY] for documentation
    CrossOriginEmbedderPolicy,

    /// Refer to [Header::CROSS_ORIGIN_OPENER_POLICY] for documentation
    CrossOriginOpenerPolicy,

    /// Refer to [Header::CROSS_ORIGIN_RESOURCE_POLICY] for documentation
    CrossOriginResourcePolicy,

    /// Refer to [Header::CONTENT_SECURITY_POLICY] for documentation
    ContentSecurityPolicy,

    /// Refer to [Header::CONTENT_SECURITY_REPORT_ONLY] for documentation
    ContentSecurityPolicyReportOnly,

    /// Refer to [Header::PERMISSIONS_POLICY] for documentation
    PermissionsPolicy,

    /// Refer to [Header::STRICT_TRANSPORT_SECURITY] for documentation
    StrictTransportSecurity,

    /// Refer to [Header::UPGRADE_INSECURE_REQUESTS] for documentation
    UpgradeInsecureRequests,

    /// Refer to [Header::CONTENT_TYPE_OPTIONS] for documentation
    ContentTypeOptions,

    /// Refer to [Header::FRAME_OPTIONS] for documentation
    FrameOptions,

    /// Refer to [Header::PERMITTED_CROSS_DOMAIN_POLICIES] for documentation
    PermittedCrossDomainPolicies,

    /// Refer to [Header::POWERED_BY] for documentation
    PoweredBy,

    /// Refer to [Header::XSS_PROTECTION] for documentation
    XssProtection,

    /// Refer to [Header::SEC_FETCH_SITE] for documentation
    SecFetchSite,

    /// Refer to [Header::SEC_FETCH_MODE] for documentation
    SecFetchMode,

    /// Refer to [Header::SEC_FETCH_USER] for documentation
    SecFetchUser,

    /// Refer to [Header::SEC_FETCH_DEST] for documentation
    SecFetchDest,

    /// Refer to [Header::SEC_PURPOSE] for documentation
    SecPurpose,

    /// Refer to [Header::SERVICE_WORKER_NAVIGATION_PRELOAD] for documentation
    ServiceWorkerNavigationPreload,

    /// Refer to [Header::REPORT_TO] for documentation
    ReportTo,

    /// Refer to [Header::TRANSFER_ENCODING] for documentation
    TransferEncoding,

    /// Refer to [Header::TE] for documentation
    Te,

    /// Refer to [Header::TRAILER] for documentation
    Trailer,

    /// Refer to [Header::ALT_SVC] for documentation
    AltSvc,

    /// Refer to [Header::ALT_USED] for documentation
    AltUsed,

    /// Refer to [Header::DATE] for documentation
    Date,

    /// Refer to [Header::LINK] for documentation
    Link,

    /// Refer to [Header::RETRY_AFTER] for documentation
    RetryAfter,

    /// Refer to [Header::SERVER_TIMING] for documentation
    ServerTiming,

    /// Refer to [Header::SERVICE_WORKER_ALLOWED] for documentation
    ServiceWorkerAllowed,

    /// Refer to [Header::SOURCE_MAP] for documentation
    SourceMap,

    /// Refer to [Header::UPGRADE] for documentation
    Upgrade,

    /// Refer to [Header::ACCEPT_CH] for documentation
    AcceptClientHints,

    /// Refer to [Header::CRITICAL_CH] for documentation
    CriticalClientHints,

    /// Refer to [Header::SEC_CH_UA] for documentation
    SecClientHintUserAgent,

    /// Refer to [Header::SEC_CH_UA_ARCH] for documentation
    SecClientHintUserAgentArchitecture,

    /// Refer to [Header::SEC_CH_UA_ARCH] for documentation
    SecClientHintUserAgentBitness,

    /// Refer to [Header::SEC_CH_UA_FULL_VERSION_LIST] for documentation
    SecClientHintUserAgentFullVersionList,

    /// Refer to [Header::SEC_CH_UA_MOBILE] for documentation
    SecClientHintUserAgentMobile,

    /// Refer to [Header::SEC_CH_UA_MODEL] for documentation
    SecClientHintUserAgentModel,

    /// Refer to [Header::SEC_CH_UA_PLATFORM] for documentation
    SecClientHintUserAgentPlatform,

    /// Refer to [Header::SEC_CH_UA_PLATFORM_VERSION] for documentation
    SecClientHintUserAgentPlatformVersion,

    /// Refer to [Header::SEC_CH_UA_PREFERS_COLOR_SCHEME] for documentation
    SecClientHintUserAgentPrefersColorScheme,

    /// Refer to [Header::SEC_CH_UA_PREFERS_REDUCED_MOTION] for documetation
    SecClientHintUserAgentPrefersReducedMotion,

    /// Refer to [Header::DEVICE_MEMORY] for documentation
    DeviceMemory,

    NonStandard(NonStandardHeader),
}

/// Headers that are not standardized, but still common around the web
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NonStandardHeader {
    /// Refer to [Header::X_FORWARDED_FOR] for documentation
    ForwardedFor,

    /// Refer to [Header::X_FORWARDED_HOST] for documentation
    ForwardedHost,

    /// Refer to [Header::X_FORWARDED_PROTO] for documentation
    ForwardedProto,

    /// Refer to [Header::X_DNS_PREFETCH_CONTROL] for documentation
    DnsPrefetchControl,
}
