use std::time;

use crate::{
    css::{layout::flow::BlockFormattingContext, StyleComputer},
    html::{self, tokenization::IgnoreParseErrors},
};

/// The Browsing Context takes care of coordinating loads, layout calculations and paints
pub struct BrowsingContext;

#[derive(Debug)]
pub enum BrowsingContextError {
    Loading(mime::ResourceLoadError),
    UnsupportedMIME,
}

impl BrowsingContext {
    pub fn load(location: &str) -> Result<Self, BrowsingContextError> {
        // Load the content at the given url
        let resource = mime::Resource::load(location).map_err(BrowsingContextError::Loading)?;

        if !resource.metadata.computed_mime_type.is_html() {
            log::error!(
                "Cannot display unknown MIME type: {}",
                resource.metadata.computed_mime_type
            );
            return Err(BrowsingContextError::UnsupportedMIME);
        }
        // FIXME: resource might not be utf-8
        let html_source = String::from_utf8_lossy(&resource.data);

        // Parse the data into a html document
        let parse_start = time::Instant::now();
        let parser: html::Parser<IgnoreParseErrors> = html::Parser::new(&html_source);
        let (document, stylesheets) = parser.parse();
        let parse_end = time::Instant::now();

        log::info!(
            "Parsed document in {}ms",
            parse_end.duration_since(parse_start).as_millis()
        );
        log::info!("{:?}", document);
        log::info!("Found {} stylesheets, {stylesheets:?}", stylesheets.len());
        let style_computer = StyleComputer::new(&stylesheets);

        // Build a box tree for the parsed document
        let box_tree = BlockFormattingContext::root(document, style_computer);
        log::info!("box tree: \n{box_tree:?}");

        Ok(Self)
    }
}
