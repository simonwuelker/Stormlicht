use error_derive::Error;
use http::request::HTTPError;
use sl_std::{ascii, base64};
use std::{fs, io};
use url::URL;

#[derive(Clone, Debug)]
pub struct Resource {
    data: Vec<u8>,
    mime_metadata: mime::Metadata,
    protocol_specific_data: ProtocolSpecificData,
}

#[derive(Clone, Debug)]
enum ProtocolSpecificData {
    Http(http::Headers),

    /// The protocol did not supply is with additional relevant information
    None,
}

#[derive(Debug, Error)]
pub enum ResourceLoadError {
    #[msg = "http request failed"]
    HTTP(HTTPError),

    #[msg = "invalid base64"]
    Base64(base64::Error),

    #[msg = "unsupported url scheme"]
    UnsupportedScheme,

    #[msg = "invalid file path"]
    InvalidFilePath,

    #[msg = "invalid data url"]
    InvalidDataURL,

    #[msg = "io error"]
    IO(io::Error),
}

impl Resource {
    #[must_use]
    pub fn new_for_http_request(data: Vec<u8>, headers: http::Headers) -> Self {
        let mime_metadata = mime::Metadata::for_http_request(&data, &headers, mime::NoSniff::No);

        Self {
            data,
            mime_metadata,
            protocol_specific_data: ProtocolSpecificData::Http(headers),
        }
    }

    #[must_use]
    pub fn new(data: Vec<u8>, mimetype_hint: Option<mime::MIMEType>) -> Self {
        let mime_metadata =
            mime::Metadata::with_supplied_mime_type(&data, mimetype_hint, mime::NoSniff::No);

        Self {
            data,
            mime_metadata,
            protocol_specific_data: ProtocolSpecificData::None,
        }
    }

    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    #[must_use]
    pub fn mime_metadata(&self) -> &mime::Metadata {
        &self.mime_metadata
    }

    /// Return the response headers associated with this response, or `None`
    /// if the [Resource] was not transferred over HTTP.
    #[must_use]
    pub fn http_headers(&self) -> Option<&http::Headers> {
        match &self.protocol_specific_data {
            ProtocolSpecificData::Http(headers) => Some(headers),
            _ => None,
        }
    }

    pub fn load(url: &URL) -> Result<Resource, ResourceLoadError> {
        log::info!(
            "Starting load of {}",
            url.serialize(url::ExcludeFragment::Yes)
        );

        let resource = match url.scheme().as_str() {
            "http" | "https" => {
                // Fetch the file via http
                let response = http::request::Request::get(url).send()?;

                Self::new_for_http_request(response.body, response.headers)
            },
            "file" => {
                // Fetch the file from the local filesystem
                let data = match url.as_file_path() {
                    Ok(path) => fs::read(path)?,
                    Err(_) => {
                        log::error!(
                            "Failed to load {}: Invalid file path for current platform",
                            url.serialize(url::ExcludeFragment::Yes)
                        );
                        return Err(ResourceLoadError::InvalidFilePath);
                    },
                };

                Self::new(data, None)
            },
            "data" => {
                // Load data encoded directly in the URL
                // https://www.rfc-editor.org/rfc/rfc2397#section-2
                if !url.has_opaque_path() {
                    log::error!(
                        "Failed to load {}: data URLs need to have an opaque path",
                        url.serialize(url::ExcludeFragment::Yes)
                    );
                    return Err(ResourceLoadError::InvalidDataURL);
                }

                let opaque_path = &url.path();
                let (mut before_data, data) = match opaque_path.split_once(ascii::Char::Comma) {
                    Some(segments) => segments,
                    None => return Err(ResourceLoadError::InvalidDataURL),
                };

                let is_b64 = if before_data.as_bytes().ends_with(b";base64") {
                    before_data = &before_data[..before_data.len() - b";base64".len()];
                    true
                } else {
                    false
                };

                let mut supplied_mime_type = None;
                if !before_data.is_empty() {
                    // We treat parse errors in the provided mime type as if no mime type
                    // had been provided
                    supplied_mime_type = before_data.as_str().parse().ok();
                }

                let data = if is_b64 {
                    base64::b64decode(data)?
                } else {
                    url::percent_decode(data).to_vec()
                };

                Self::new(data, supplied_mime_type)
            },
            other => {
                log::error!(
                    "Failed to load unknown url scheme: {other} from {}",
                    url.serialize(url::ExcludeFragment::Yes)
                );
                return Err(ResourceLoadError::UnsupportedScheme);
            },
        };

        log::info!(
            "Successfully loaded {}",
            url.serialize(url::ExcludeFragment::Yes)
        );

        Ok(resource)
    }
}
