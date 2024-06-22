use std::time;

use render::Composition;
use resourceloader::{ResourceLoadError, RESOURCE_LOADER};
use url::URL;

use crate::{
    css::{
        display_list::Painter,
        fragment_tree::FragmentTree,
        layout::{BoxTree, Pixels, Size},
        StyleComputer, Stylesheet,
    },
    dom::{dom_objects, DomPtr},
    event,
    html::{self, tokenization::IgnoreParseErrors},
};

/// The Browsing Context takes care of coordinating loads, layout calculations and paints
#[derive(Default)]
pub struct BrowsingContext {
    /// The currently loaded web page, or none if no page is loaded
    current_page: Option<CurrentPage>,
}

struct CurrentPage {
    document: DomPtr<dom_objects::Document>,
    fragment_tree: FragmentTree,
    stylesheets: Vec<Stylesheet>,
    hovered_element: Option<DomPtr<dom_objects::Element>>,
    needs_relayout: bool,
}

#[derive(Debug)]
pub enum BrowsingContextError {
    Loading(ResourceLoadError),
    UnsupportedMIME,
}

impl BrowsingContext {
    pub fn load(&mut self, location: &URL) -> Result<(), BrowsingContextError> {
        // Load the content at the given url
        let resource = RESOURCE_LOADER
            .schedule_load(location.clone())
            .block()
            .map_err(BrowsingContextError::Loading)?;

        if !resource.mime_metadata().computed_mime_type.is_html() {
            log::error!(
                "Cannot display unknown MIME type: {}",
                resource.mime_metadata().computed_mime_type
            );
            return Err(BrowsingContextError::UnsupportedMIME);
        }

        // FIXME: resource might not be utf-8
        let html_source = String::from_utf8_lossy(&resource.data());

        // Parse the data into a html document
        let parse_start = time::Instant::now();
        let parser: html::Parser<IgnoreParseErrors> = html::Parser::new(&html_source);
        let (document, stylesheets) = parser.parse();
        let parse_end = time::Instant::now();

        log::info!(
            "Parsed document in {}ms",
            parse_end.duration_since(parse_start).as_millis()
        );

        let current_page = CurrentPage {
            document,
            fragment_tree: FragmentTree::default(),
            stylesheets,
            hovered_element: None,
            needs_relayout: true,
        };

        self.current_page = Some(current_page);

        Ok(())
    }

    pub fn paint(&mut self, to: &mut Composition, viewport_size: (u16, u16)) {
        let Some(current_page) = &mut self.current_page else {
            return;
        };

        let viewport_size = Size {
            width: Pixels(viewport_size.0 as f32),
            height: Pixels(viewport_size.1 as f32),
        };

        if current_page.needs_relayout {
            current_page.layout(viewport_size);
        }

        // Paint the fragment_tree to the screen
        let mut painter = Painter::default();
        current_page
            .fragment_tree
            .fill_display_list(&mut painter, viewport_size);

        painter.paint(to);
    }

    pub fn handle_mouse_event(&mut self, mouse_event: event::MouseEvent) {
        let Some(current_page) = &mut self.current_page else {
            return;
        };

        let mouse_position = mouse_event.position.map(|x| Pixels(x as f32));

        let hovered_element: Option<DomPtr<dom_objects::Element>> = current_page
            .fragment_tree
            .hit_test(mouse_position)
            .and_then(|fragment| fragment.dom_node())
            .and_then(|node| node.try_into_type());

        current_page.update_hovered_element(hovered_element);
    }
}

impl CurrentPage {
    fn layout(&mut self, viewport_size: Size<Pixels>) {
        let layout_start = time::Instant::now();
        let style_computer = StyleComputer::new(&self.stylesheets, Pixels(16.), viewport_size);

        // Build a box tree for the parsed document
        let box_tree = BoxTree::new(self.document.clone(), style_computer);
        log::info!("\n{:?}", box_tree);

        // Build a fragment tree by fragmenting the boxes
        self.fragment_tree = box_tree.compute_fragments(viewport_size);

        let layout_end = time::Instant::now();
        log::info!(
            "Layout took {}ms",
            layout_end.duration_since(layout_start).as_millis()
        );

        self.needs_relayout = false;
    }

    fn update_hovered_element(&mut self, hovered_element: Option<DomPtr<dom_objects::Element>>) {
        // Update hover state and invalidate layout if necessary
        match (hovered_element.clone(), self.hovered_element.clone()) {
            (Some(new_element), Some(old_element)) => {
                if new_element.ptr_eq(&old_element) {
                    // Nothing to do
                    return;
                }
                old_element.borrow_mut().set_hovered(false);
                new_element.borrow_mut().set_hovered(true);
            },
            (Some(new_element), None) => {
                new_element.borrow_mut().set_hovered(true);
            },
            (None, Some(old_element)) => {
                old_element.borrow_mut().set_hovered(false);
            },
            (None, None) => return,
        }
        self.hovered_element = hovered_element;

        // Changing the hovered element can change the CSS rules that apply (via the :hover pseudoclass)
        // and therefore invalidates layout
        self.invalidate_layout();
    }

    fn invalidate_layout(&mut self) {
        self.needs_relayout = true;
    }
}
