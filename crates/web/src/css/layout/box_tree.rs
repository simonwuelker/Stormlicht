use std::fmt;

use math::Vec2D;

use crate::{
    css::{
        computed_style::ComputedStyle,
        fragment_tree::FragmentTree,
        layout::{ContainingBlock, Pixels, Size},
        StyleComputer,
    },
    dom::{dom_objects, DomPtr},
    TreeDebug, TreeFormatter,
};

use super::flow::{BlockContainerBuilder, BlockFormattingContext};

#[derive(Clone)]
pub struct BoxTree {
    /// The root box acts like it's inside a [BlockFormattingContext](super::flow::BlockFormattingContext), except
    /// that the formatting context always only contains a single element (said root box) and the size of
    /// the root box is always equivalent to the viewport.
    ///
    /// There might be *no* root boxes if the root element has `display: none;`
    // FIXME: can there be more than one root element?
    root: BlockFormattingContext,
}

impl BoxTree {
    #[must_use]
    pub fn new(document: DomPtr<dom_objects::Document>, style_computer: StyleComputer<'_>) -> Self {
        let html = document
            .borrow()
            .children()
            .last()
            .expect("no root element found")
            .try_into_type::<dom_objects::HtmlHtmlElement>()
            .expect("expected root element to be html element");

        let parent_style = ComputedStyle::default();
        let element_style = style_computer.get_computed_style(html.clone().upcast(), &parent_style);

        let mut container = BlockContainerBuilder::new(&parent_style, style_computer);

        container.handle_element(html.upcast(), element_style);

        let contents = container.finish();

        Self {
            root: contents.into(),
        }
    }

    pub fn compute_fragments(&self, viewport: Size<Pixels>) -> FragmentTree {
        // The initial containing block always has the size of the viewport
        let initial_containing_block =
            ContainingBlock::new(viewport.width, Vec2D::new(Pixels::ZERO, Pixels::ZERO))
                .with_height(viewport.height);
        let content_info = self.root.layout(initial_containing_block);

        FragmentTree::new(content_info.fragments)
    }
}

impl fmt::Debug for BoxTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tree_formatter = TreeFormatter::new(f);
        self.root.tree_fmt(&mut tree_formatter)
    }
}
