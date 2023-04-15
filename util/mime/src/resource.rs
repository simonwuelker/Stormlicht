//! <https://mimesniff.spec.whatwg.org/#resource>

use std::str::FromStr;

use crate::{sniff, MIMEType};

use http::request::HTTPError;
use url::URL;

/// Whether or not the user agent should try to guess the computed [MIMEType] of a [Resource].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum NoSniff {
    #[default]
    Yes,
    No,
}

/// Whether or not the user agent should check for an apache bug
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
    BadURL(url::URLParseError),
}

impl Resource {
    pub fn load(source: &str) -> Result<Resource, ResourceLoadError> {
        let url = URL::parse(source).map_err(ResourceLoadError::BadURL)?;
        Self::load_url(url)
    }

    pub fn load_url(url: URL) -> Result<Resource, ResourceLoadError> {
        log::info!(
            "Starting load of {}",
            url.serialize(url::ExcludeFragment::Yes)
        );

        let mut supplied_type = None;
        let mut check_for_apache_bug = CheckForApacheBug::default();

        let data = match url.scheme.as_str() {
            "http" => {
                // Fetch the file via http
                let response = http::request::Request::get(url)
                    .send()
                    .map_err(ResourceLoadError::HTTP)?;

                if let Some(content_type_string) = response.headers.get("Content-Type") {
                    if let Ok(content_type) = MIMEType::from_str(content_type_string) {
                        supplied_type = Some(content_type);
                    }

                    if matches!(
                        content_type_string.as_str(),
                        "text/plain"
                            | "text/plain; charset=ISO-8859-1"
                            | "text/plain; charset=iso-8859-1"
                            | "text/plain; charset=UTF-8"
                    ) {
                        check_for_apache_bug = CheckForApacheBug::Yes;
                    }
                }

                response.body
            },
            "file" => {
                // Fetch the file from the local filesystem
                todo!()
            },
            other => {
                log::error!(
                    "Failed to load unknown url scheme: {other} from {}",
                    url.serialize(url::ExcludeFragment::Yes)
                );
                return Err(ResourceLoadError::UnsupportedScheme);
            },
        };

        let metadata = ResourceMetadata::new(
            supplied_type,
            check_for_apache_bug,
            NoSniff::default(),
            &data,
        );

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
        let computed_mime_type =
            determine_computed_mimetype(supplied_mime_type.as_ref(), no_sniff, resource_data);

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
    resource_data: &[u8],
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

        return sniff::identify_unknown_mime_type(resource_data, sniff_scriptable);
    }

    // 2. If the no-sniff flag is set, the computed MIME type is the supplied MIME type.
    if no_sniff == NoSniff::Yes {
        // Abort these steps.
        return supplied_mime_type.unwrap().clone();
    }

    todo!()
}
