use std::fmt;

use math::Vec2D;

use crate::{
    css::{
        computed_style::ComputedStyle,
        font_metrics::DEFAULT_FONT_SIZE,
        fragment_tree::FragmentTree,
        layout::{
            flow::{BlockFlowState, BlockLevelBox, BoxTreeBuilder},
            ContainingBlock, Pixels, Size,
        },
        values::length,
        StyleComputer,
    },
    dom::{dom_objects, DomPtr},
    TreeDebug, TreeFormatter,
};

use super::flow::{BlockFormattingContext, InFlowBlockBox};

#[derive(Clone)]
pub struct BoxTree {
    /// The root box acts like it's inside a [BlockFormattingContext](super::flow::BlockFormattingContext), except
    /// that the formatting context always only contains a single element (said root box) and the size of
    /// the root box is always equivalent to the viewport.
    ///
    /// There might be *no* root boxes if the root element has `display: none;`
    // FIXME: can there be more than one root element?
    root: Vec<BlockLevelBox>,
}

impl BoxTree {
    pub fn new(document: DomPtr<dom_objects::Document>, style_computer: StyleComputer<'_>) -> Self {
        let html = document
            .borrow()
            .children()
            .last()
            .expect("no root element found")
            .try_into_type::<dom_objects::HtmlHtmlElement>()
            .expect("expected root element to be html element");

        let element_style =
            style_computer.get_computed_style(html.clone().upcast(), &ComputedStyle::default());

        let contents = BoxTreeBuilder::build(
            DomPtr::clone(&html).upcast(),
            style_computer,
            &element_style,
        );

        let root_box = InFlowBlockBox::new(element_style, Some(html.upcast()), contents).into();

        Self {
            root: vec![root_box],
        }
    }
    pub fn compute_fragments(&self, viewport: Size<Pixels>) -> FragmentTree {
        // The root box always has the size of the viewport
        let origin = Vec2D::new(Pixels::ZERO, Pixels::ZERO);
        let initial_containing_block =
            ContainingBlock::new(origin, viewport.width).with_height(viewport.height);
        let length_resolution_context = length::ResolutionContext {
            font_size: DEFAULT_FONT_SIZE,
            root_font_size: DEFAULT_FONT_SIZE,
            viewport,
        };

        let mut root_formatting_context = BlockFormattingContext::default();
        let mut state = BlockFlowState::new(
            origin,
            initial_containing_block,
            length_resolution_context,
            &mut root_formatting_context,
        );
        for root_box in &self.root {
            state.visit_block_box(root_box);
        }

        let (_height, root_fragments) = state.finish();

        FragmentTree::new(root_fragments)
    }
}

impl fmt::Debug for BoxTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tree_formatter = TreeFormatter::new(f);
        for root_box in &self.root {
            root_box.tree_fmt(&mut tree_formatter)?;
        }
        Ok(())
    }
}
