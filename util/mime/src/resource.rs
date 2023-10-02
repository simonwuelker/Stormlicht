//! <https://mimesniff.spec.whatwg.org/#resource>

use std::{fs, io, str::FromStr};

use crate::{
    sniff::{self, identify_audio_or_video_type, identify_image_type},
    sniff_tables, MIMEType,
};

use http::request::HTTPError;
use url::{ExcludeFragment, URL};

/// Whether or not the user agent should try to guess the computed [MIMEType] of a [Resource].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoSniff {
    Yes,
    No,
}

/// Whether or not the user agent should check for an [apache bug](https://issues.apache.org/bugzilla/show_bug.cgi?id=13986)
/// that caused apache to send unexpected `Content-Type` HTTP Headers when serving files with an unknown
/// MIME Type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CheckForApacheBug {
    Yes,
    #[default]
    No,
}

/// Whether or not the user agent should try to guess scriptable [MIMETypes](MIMEType)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SniffScriptable {
    Yes,
    No,
}

/// <https://mimesniff.spec.whatwg.org/#resource>
#[derive(Clone, Debug)]
pub struct ResourceMetadata {
    pub supplied_mime_type: Option<MIMEType>,
    pub computed_mime_type: MIMEType,
    pub check_for_apache_bug: CheckForApacheBug,
    pub no_sniff: NoSniff,
}

#[derive(Clone, Debug)]
pub struct Resource {
    pub metadata: ResourceMetadata,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum ResourceLoadError {
    HTTP(HTTPError),
    UnsupportedScheme,
    IO(io::Error),
}

impl From<HTTPError> for ResourceLoadError {
    fn from(value: HTTPError) -> Self {
        Self::HTTP(value)
    }
}

impl From<io::Error> for ResourceLoadError {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}

impl Resource {
    pub fn load(url: &URL) -> Result<Resource, ResourceLoadError> {
        log::info!(
            "Starting load of {}",
            url.serialize(url::ExcludeFragment::Yes)
        );

        let mut supplied_type = None;
        let mut check_for_apache_bug = CheckForApacheBug::default();

        let data = match url.scheme().as_str() {
            "http" | "https" => {
                // Fetch the file via http
                let response = http::request::Request::get(url).send()?;
                log::info!(
                    "Successfully loaded {}",
                    response.context().url.serialize(ExcludeFragment::Yes)
                );

                if let Some(content_type_string) = response.headers().get("Content-Type") {
                    if let Ok(content_type) = MIMEType::from_str(content_type_string) {
                        supplied_type = Some(content_type);
                    }

                    if matches!(
                        content_type_string,
                        "text/plain"
                            | "text/plain; charset=ISO-8859-1"
                            | "text/plain; charset=iso-8859-1"
                            | "text/plain; charset=UTF-8"
                    ) {
                        check_for_apache_bug = CheckForApacheBug::Yes;
                    }
                }

                response.into_body()
            },
            "file" => {
                // Fetch the file from the local filesystem
                // FIXME: make this cross-platform compatible
                let mut path = String::new();
                for segment in url.path() {
                    path.push('/');
                    path.push_str(segment.as_str());
                }

                fs::read(path)?
            },
            other => {
                log::error!(
                    "Failed to load unknown url scheme: {other} from {}",
                    url.serialize(url::ExcludeFragment::Yes)
                );
                return Err(ResourceLoadError::UnsupportedScheme);
            },
        };

        let metadata =
            ResourceMetadata::new(supplied_type, check_for_apache_bug, NoSniff::No, &data);

        Ok(Resource { metadata, data })
    }
}

impl ResourceMetadata {
    pub fn new(
        supplied_mime_type: Option<MIMEType>,
        check_for_apache_bug: CheckForApacheBug,
        no_sniff: NoSniff,
        resource_data: &[u8],
    ) -> Self {
        let computed_mime_type = determine_computed_mimetype(
            supplied_mime_type.as_ref(),
            no_sniff,
            check_for_apache_bug,
            resource_data,
        );

        Self {
            supplied_mime_type,
            computed_mime_type,
            check_for_apache_bug,
            no_sniff,
        }
    }
}

/// <https://mimesniff.spec.whatwg.org/#determining-the-computed-mime-type-of-a-resource>
pub fn determine_computed_mimetype(
    supplied_mime_type: Option<&MIMEType>,
    no_sniff: NoSniff,
    check_for_apache_bug: CheckForApacheBug,
    resource_header: &[u8],
) -> MIMEType {
    // 1. If the supplied MIME type is undefined or if the supplied MIME type’s essence is "unknown/unknown", "application/unknown", or "*/*",
    // execute the rules for identifying an unknown MIME type with the sniff-scriptable flag equal to the inverse of the no-sniff flag and abort these steps.
    if supplied_mime_type.is_none()
        || supplied_mime_type.as_ref().is_some_and(|mime_type| {
            matches!(
                mime_type.essence().as_str(),
                "unknown/unknown" | "application/unknown" | "*/*"
            )
        })
    {
        let sniff_scriptable = match no_sniff {
            NoSniff::Yes => SniffScriptable::No,
            NoSniff::No => SniffScriptable::Yes,
        };

        return sniff::identify_unknown_mime_type(resource_header, sniff_scriptable);
    }
    let supplied_mime_type =
        supplied_mime_type.expect("validated that supplied_mime_type is not none in step 1.");

    // 2. If the no-sniff flag is set, the computed MIME type is the supplied MIME type.
    if no_sniff == NoSniff::Yes {
        // Abort these steps.
        return supplied_mime_type.clone();
    }

    // 3. If the check-for-apache-bug flag is set, execute the rules for distinguishing if a resource is text or binary and abort these steps.
    if check_for_apache_bug == CheckForApacheBug::Yes {
        // https://mimesniff.spec.whatwg.org/#rules-for-text-or-binary

        // 1. Let length be the number of bytes in the resource header.
        let length = resource_header.len();

        // 2. If length is greater than or equal to 2 and the first 2 bytes of the resource header
        // are equal to 0xFE 0xFF (UTF-16BE BOM) or 0xFF 0xFE (UTF-16LE BOM), the computed MIME type is "text/plain".
        if length >= 2
            && (resource_header[..2] == [0xFE, 0xFF] || resource_header[..2] == [0xFF, 0xFE])
        {
            return MIMEType::new("text", "plain");
        }

        // 3. If length is greater than or equal to 3 and the first 3 bytes of the resource header are equal to 0xEF 0xBB 0xBF (UTF-8 BOM), the computed MIME type is "text/plain".
        if length >= 3 && resource_header[..3] == [0xEF, 0xBB, 0xBF] {
            return MIMEType::new("text", "plain");
        }

        // 4. If the resource header contains no binary data bytes, the computed MIME type is "text/plain".
        if resource_header
            .iter()
            .all(|&byte| !sniff::is_binary_data_byte(byte))
        {
            return MIMEType::new("text", "plain");
        }

        // 5. The computed MIME type is "application/octet-stream".
        return MIMEType::new("application", "octet-stream");
    }

    // 4. If the supplied MIME type is an XML MIME type, the computed MIME type is the supplied MIME type.
    if supplied_mime_type.is_xml() {
        return supplied_mime_type.clone();
    }

    // 5. If the supplied MIME type’s essence is "text/html", execute the rules for distinguishing if a resource is a feed or HTML and abort these steps.
    if supplied_mime_type.essence() == "text/html" {
        // https://mimesniff.spec.whatwg.org/#rules-for-distinguishing-if-a-resource-is-a-feed-or-html

        // 1. Let sequence be the resource header, where sequence[s] is byte s in sequence and sequence[0] is the first byte in sequence.
        let sequence = resource_header;

        // 2. Let length be the number of bytes in sequence.
        let length = sequence.len();

        // 3. Initialize s to 0.
        let mut s = 0;

        // 4. If length is greater than or equal to 3 and the three bytes from sequence[0] to sequence[2] are equal to 0xEF 0xBB 0xBF (UTF-8 BOM), increment s by 3.
        if sequence.starts_with(&[0xEF, 0xBB, 0xBF]) {
            s += 3;
        }

        // 5. While s is less than length, continuously loop through these steps:
        'outer_loop: while s < length {
            // 1. Enter loop L:
            'L: loop {
                match sequence.get(s) {
                    None => {
                        // 1. If sequence[s] is undefined, the computed MIME type is the supplied MIME type.
                        // Abort these steps.
                        return supplied_mime_type.clone();
                    },
                    Some(0x3C) => {
                        // 2. If sequence[s] is equal to 0x3C ("<"), increment s by 1 and exit loop L.
                        s += 1;
                        break 'L;
                    },
                    Some(byte) if !sniff_tables::WHITESPACE.contains(byte) => {
                        // 3. If sequence[s] is not a whitespace byte, the computed MIME type is the supplied MIME type.
                        // Abort these steps.
                        return supplied_mime_type.clone();
                    },
                    Some(_) => {},
                }

                // 4. Increment s by 1.
                s += 1;
            }

            // 2. Enter loop L:
            // NOTE: this seems to be a spec bug, theres no way for the loop to run more than once.
            // See https://github.com/whatwg/mimesniff/issues/169
            {
                // 1. If sequence[s] is undefined, the computed MIME type is the supplied MIME type.
                // Abort these steps.
                if length <= s {
                    return supplied_mime_type.clone();
                }

                // 2. If length is greater than or equal to s + 3 and the three bytes from sequence[s] to sequence[s + 2] are equal to 0x21 0x2D 0x2D ("!--"),
                // increment s by 3 and enter loop M:
                if sequence[s..].starts_with(b"!--") {
                    s += 3;

                    loop {
                        // 1. If sequence[s] is undefined, the computed MIME type is the supplied MIME type.
                        // Abort these steps.
                        if length <= s {
                            return supplied_mime_type.clone();
                        }

                        // 2. If length is greater than or equal to s + 3 and the three bytes from sequence[s] to sequence[s + 2]
                        // are equal to 0x2D 0x2D 0x3E ("-->"), increment s by 3 and exit loops M and L.
                        if sequence[s..].starts_with(b"-->") {
                            s += 3;

                            // NOTE: we don't have L but since the L is the last step of outer_loop we can continue
                            continue 'outer_loop;
                        }

                        // 3. Increment s by 1.
                        s += 1;
                    }
                }

                // 3. If length is greater than or equal to s + 1 and sequence[s] is equal to 0x21 ("!"), increment s by 1 and enter loop M:
                if sequence[s..].starts_with(b"!") {
                    s += 1;

                    loop {
                        // 1. If sequence[s] is undefined, the computed MIME type is the supplied MIME type.
                        // Abort these steps.
                        if length <= s {
                            return supplied_mime_type.clone();
                        }

                        // 2. If length is greater than or equal to s + 1 and sequence[s] is equal to 0x3E (">"), increment s by 1 and exit loops M and L.
                        if sequence[s..].starts_with(b">") {
                            s += 1;

                            // NOTE: we don't have L but since the L is the last step of outer_loop we can continue
                            continue 'outer_loop;
                        }

                        // 3. Increment s by 1.
                        s += 1;
                    }
                }

                // 4. If length is greater than or equal to s + 1 and sequence[s] is equal to 0x3F ("?"), increment s by 1 and enter loop M:
                if sequence[s..].starts_with(b"?") {
                    s += 1;

                    loop {
                        // 1. If sequence[s] is undefined, the computed MIME type is the supplied MIME type.
                        // Abort these steps.
                        if length <= s {
                            return supplied_mime_type.clone();
                        }

                        // 2. If length is greater than or equal to s + 2 and the two bytes from sequence[s] to sequence[s + 1] are equal to 0x3F 0x3E ("?>"), increment s by 2 and exit loops M and L.
                        if sequence[s..].starts_with(b"?>") {
                            s += 2;

                            // NOTE: we don't have L but since the L is the last step of outer_loop we can continue
                            continue 'outer_loop;
                        }

                        // 3. Increment s by 1.
                        s += 1;
                    }
                }

                // 5. If length is greater than or equal to s + 3 and the three bytes from sequence[s] to sequence[s + 2]
                // are equal to 0x72 0x73 0x73 ("rss"), the computed MIME type is "application/rss+xml".
                // Abort these steps.
                if sequence[s..].starts_with(b"rss") {
                    return MIMEType::new("application", "rss");
                }

                // 6. If length is greater than or equal to s + 4 and the four bytes from sequence[s] to sequence[s + 3] are equal to 0x66 0x65 0x65 0x64 ("feed"), the computed MIME type is "application/atom+xml".
                // Abort these steps.
                if sequence[s..].starts_with(b"feed") {
                    return MIMEType::new("application", "atom+xml");
                }

                // 7. If length is greater than or equal to s + 7 and the seven bytes from sequence[s] to sequence[s + 6] are equal to 0x72 0x64 0x66 0x3A 0x52 0x44 0x46 ("rdf:RDF"), increment s by 7 and enter loop M:
                if sequence[s..].starts_with(b"rdf:RDF") {
                    s += 7;

                    loop {
                        // 1. If sequence[s] is undefined, the computed MIME type is the supplied MIME type.
                        // Abort these steps.
                        if length <= s {
                            return supplied_mime_type.clone();
                        }

                        // 2.  If length is greater than or equal to s + 24 and the twenty-four bytes from sequence[s] to sequence[s + 23]
                        // are equal to 0x68 0x74 0x74 0x70 0x3A 0x2F 0x2F 0x70 0x75 0x72 0x6C 0x2E 0x6F 0x72 0x67 0x2F 0x72 0x73 0x73 0x2F 0x31 0x2E 0x30 0x2F
                        // ("http://purl.org/rss/1.0/"), increment s by 24 and enter loop N:
                        if sequence[s..].starts_with(b"http://purl.org/rss/1.0/") {
                            s += 24;

                            loop {
                                // 1. If sequence[s] is undefined, the computed MIME type is the supplied MIME type.
                                // Abort these steps.
                                if length <= s {
                                    return supplied_mime_type.clone();
                                }

                                // 2. If length is greater than or equal to s + 43 and the forty-three bytes from sequence[s] to sequence[s + 42]
                                // are equal to "http://www.w3.org/1999/02/22-rdf-syntax-ns#", the computed MIME type is "application/rss+xml".
                                // Abort these steps.
                                if sequence[s..]
                                    .starts_with(b"http://www.w3.org/1999/02/22-rdf-syntax-ns#")
                                {
                                    return MIMEType::new("application", "rss+xml");
                                }

                                // 3. Increment s by 1.
                                s += 1;
                            }
                        }

                        // 3. If length is greater than or equal to s + 24 and the twenty-four bytes from sequence[s] to sequence[s + 23] are equal to
                        // "http://www.w3.org/1999/02/22-rdf-syntax-ns#", increment s by 24 and enter loop N:
                        if sequence[s..].starts_with(b"http://www.w3.org/1999/02/22-rdf-syntax-ns#")
                        {
                            s += 24;

                            loop {
                                // 1. If sequence[s] is undefined, the computed MIME type is the supplied MIME type.
                                // Abort these steps.
                                if length <= s {
                                    return supplied_mime_type.clone();
                                }

                                // 2. If length is greater than or equal to s + 43 and the forty-three bytes from sequence[s] to sequence[s + 42] are
                                // equal to "http://purl.org/rss/1.0/", the computed MIME type is "application/rss+xml".
                                // Abort these steps.
                                if sequence.starts_with(b"http://purl.org/rss/1.0/") {
                                    return MIMEType::new("application", "rss+xml");
                                }

                                // 3. Increment s by 1.
                                s += 1;
                            }
                        }

                        // 4. Increment s by 1.
                        s += 1;
                    }
                }

                // 8. The computed MIME type is the supplied MIME type.
                // Abort these steps.
                return supplied_mime_type.clone();
            }
        }

        // 6. The computed MIME type is the supplied MIME type.
        return supplied_mime_type.clone();
    }

    // 6. If the supplied MIME type is an image MIME type supported by the user agent, let matched-type be the result of
    // executing the image type pattern matching algorithm with the resource header as the byte sequence to be matched.

    // 7. If matched-type is not undefined, the computed MIME type is matched-type.
    // Abort these steps.

    // NOTE: lets just act like we support all image types, i don't see any harm in that.
    if supplied_mime_type.is_image() {
        if let Some(matched_mime_type) = identify_image_type(resource_header) {
            return matched_mime_type;
        }
    }

    // 8.  If the supplied MIME type is an audio or video MIME type supported by the user agent, let matched-type be the result of
    // executing the audio or video type pattern matching algorithm with the resource header as the byte sequence to be matched.

    // 9. If matched-type is not undefined, the computed MIME type is matched-type.
    // Abort these steps.
    // NOTE: lets just act like we support all audio/video types, i don't see any harm in that.
    if supplied_mime_type.is_audio_or_video() {
        if let Some(matched_mime_type) = identify_audio_or_video_type(resource_header) {
            return matched_mime_type;
        }
    }

    // 10. The computed MIME type is the supplied MIME type.
    supplied_mime_type.clone()
}
