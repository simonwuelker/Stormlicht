use std::time;

use render::Composition;
use url::URL;

use crate::{
    css::{
        display_list::Painter,
        fragment_tree::FragmentTree,
        layout::{BoxTree, Pixels, Size},
        StyleComputer, Stylesheet,
    },
    dom::{dom_objects, DomPtr},
    html::{self, tokenization::IgnoreParseErrors},
};

/// The Browsing Context takes care of coordinating loads, layout calculations and paints
pub struct BrowsingContext {
    document: DomPtr<dom_objects::Document>,
    fragment_tree: FragmentTree,
    stylesheets: Vec<Stylesheet>,
}

#[derive(Debug)]
pub enum BrowsingContextError {
    Loading(mime::ResourceLoadError),
    UnsupportedMIME,
}

impl BrowsingContext {
    pub fn load(location: &URL) -> Result<Self, BrowsingContextError> {
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

        Ok(Self {
            document,
            fragment_tree: FragmentTree::default(),
            stylesheets,
        })
    }

    pub fn paint(&mut self, to: &mut Composition, viewport_size: (u16, u16)) {
        let layout_start = time::Instant::now();
        let style_computer = StyleComputer::new(&self.stylesheets);

        // Build a box tree for the parsed document
        let box_tree = BoxTree::new(self.document.clone(), style_computer);
        log::info!("\n{:?}", box_tree);

        // Build a fragment tree by fragmenting the boxes
        let viewport_size = Size {
            width: Pixels(viewport_size.0 as f32),
            height: Pixels(viewport_size.1 as f32),
        };
        self.fragment_tree = box_tree.compute_fragments(viewport_size);

        let layout_end = time::Instant::now();
        log::info!(
            "Layout took {}ms",
            layout_end.duration_since(layout_start).as_millis()
        );

        // Paint the fragment_tree to the screen
        let mut painter = Painter::default();
        self.fragment_tree
            .fill_display_list(&mut painter, viewport_size);

        painter.paint(to);
    }
}
