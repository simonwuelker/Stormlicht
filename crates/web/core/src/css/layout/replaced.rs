use image::Texture;
use math::{Rectangle, Vec2D};

use crate::{
    css::{
        computed_style::ComputedStyle,
        fragment_tree::{Fragment, TextureFragment},
        values::{
            length::{self, Length},
            AutoOr,
        },
    },
    dom::{dom_objects, DomPtr},
};

use super::{ContainingBlock, Pixels, Size};

/// <https://drafts.csswg.org/css2/#intrinsic>
#[derive(Clone, Copy, Debug)]
pub(crate) struct IntrinsicSize {
    pub width: Option<Pixels>,
    pub height: Option<Pixels>,

    /// `width` / `height`
    ///
    /// Note that elements may have an intrinsic aspect ratio without having an intrinsic width/height (an SVG image for example)
    pub aspect_ratio: Option<f32>,
}

impl IntrinsicSize {
    pub const NONE: Self = Self {
        width: None,
        height: None,
        aspect_ratio: None,
    };

    #[must_use]
    pub fn new(width: Pixels, height: Pixels) -> Self {
        let aspect_ratio = if width.0.is_normal() && height.0.is_normal() {
            Some(width.0 / height.0)
        } else {
            None
        };

        Self {
            width: Some(width),
            height: Some(height),
            aspect_ratio,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum ReplacedContent {
    Image(Texture<u32>),
}

/// <https://drafts.csswg.org/css-display/#replaced-element>
#[derive(Clone, Debug)]
pub(crate) struct ReplacedElement {
    pub intrinsic_size: IntrinsicSize,
    pub content: ReplacedContent,
    pub style: ComputedStyle,
}

impl ReplacedElement {
    #[must_use]
    const fn has_intrinsic_width(&self) -> bool {
        self.intrinsic_size.width.is_some()
    }

    #[must_use]
    const fn has_intrinsic_height(&self) -> bool {
        self.intrinsic_size.height.is_some()
    }

    #[must_use]
    const fn has_intrinsic_aspect_ratio(&self) -> bool {
        self.intrinsic_size.aspect_ratio.is_some()
    }

    #[must_use]
    pub fn style(&self) -> ComputedStyle {
        self.style.clone()
    }

    #[must_use]
    pub const fn content(&self) -> &ReplacedContent {
        &self.content
    }

    /// <https://drafts.csswg.org/css2/#inline-replaced-width>
    fn used_inline_width(
        &self,
        containining_block: ContainingBlock,
        length_resolution_context: length::ResolutionContext,
    ) -> Pixels {
        let computed_width = self.style.width();
        let computed_height = self.style.height();

        if let AutoOr::NotAuto(width) = computed_width {
            let available_width = Length::pixels(containining_block.width());

            width
                .resolve_against(available_width)
                .absolutize(length_resolution_context)
        } else if computed_height.is_auto()
            && let Some(intrinsic_width) = self.intrinsic_size.width
        {
            intrinsic_width
        } else if let Some(intrinsic_height) = self.intrinsic_size.height
            && let Some(intrinsic_aspect_ratio) = self.intrinsic_size.aspect_ratio
        {
            intrinsic_height * intrinsic_aspect_ratio
        } else if let AutoOr::NotAuto(height) = computed_height
            && let Some(intrinsic_aspect_ratio) = self.intrinsic_size.aspect_ratio
            && let Some(container_height) = containining_block.height()
        {
            // The spec doesn't explicitly state this, but to use the "used height" here,
            // the height of the containing block is required to be known.
            let available_height = Length::pixels(container_height);

            let used_height = height
                .resolve_against(available_height)
                .absolutize(length_resolution_context);

            used_height * intrinsic_aspect_ratio
        } else if computed_height.is_auto()
            && !self.has_intrinsic_width()
            && !self.has_intrinsic_height()
            && self.has_intrinsic_aspect_ratio()
        {
            log::warn!("Computing width of replaced element with neither height nor width but an intrinsic aspect ratio, this is undefined in CSS2");
            log::warn!("Falling back to 0.0 pixels");
            Pixels::ZERO
        } else if let Some(intrinsic_width) = self.intrinsic_size.width {
            // NOTE: This is the same case as above, but without the condition that height is auto
            intrinsic_width
        } else {
            let viewport = length_resolution_context.viewport;

            if viewport.width < Pixels(300.) {
                // The width of the largest rectangle with a 2:1 aspect ratio that fits on the viewport
                if viewport.width < viewport.height * 2. {
                    viewport.width
                } else {
                    viewport.height / 2.
                }
            } else {
                Pixels(300.)
            }
        }
    }

    /// <https://drafts.csswg.org/css2/#inline-replaced-height>
    #[must_use]
    fn used_inline_height(
        &self,
        containining_block: ContainingBlock,
        length_resolution_context: length::ResolutionContext,
    ) -> Pixels {
        let computed_width = self.style.width();
        let computed_height = self.style.height();

        if let AutoOr::NotAuto(height) = computed_height
            && let Some(available_height) = containining_block.height()
        {
            let available_height = Length::pixels(available_height);
            height
                .resolve_against(available_height)
                .absolutize(length_resolution_context)
        } else if computed_width.is_auto()
            && let Some(intrinsic_height) = self.intrinsic_size.height
        {
            intrinsic_height
        } else if let AutoOr::NotAuto(width) = computed_width
            && let Some(intrinsic_aspect_ratio) = self.intrinsic_size.aspect_ratio
        {
            let available_width = Length::pixels(containining_block.width());
            let used_width = width
                .resolve_against(available_width)
                .absolutize(length_resolution_context);

            used_width * intrinsic_aspect_ratio
        } else if let Some(intrinsic_height) = self.intrinsic_size.height {
            intrinsic_height
        } else {
            // The height of the largest rectangle that has a 2:1 ratio,
            //  a height not greater than 150px, and has a width not greater than the device width.
            let device_width = length_resolution_context.viewport.width;
            (device_width / 2.).min(Pixels(150.))
        }
    }

    /// The content size of the element, assuming it's inline
    ///
    /// See  <https://drafts.csswg.org/css2/#inline-replaced-width> and <https://drafts.csswg.org/css2/#inline-replaced-height>
    #[must_use]
    pub fn used_size_if_it_was_inline(
        &self,
        containining_block: ContainingBlock,
        length_resolution_context: length::ResolutionContext,
    ) -> Size<Pixels> {
        let width = self.used_inline_width(containining_block, length_resolution_context);
        let height = self.used_inline_height(containining_block, length_resolution_context);
        Size { width, height }
    }

    #[must_use]
    pub fn try_from(
        element: DomPtr<dom_objects::Element>,
        element_style: ComputedStyle,
    ) -> Option<Self> {
        // Check if the element is replaced
        // Currently the only replaced element supported is the <img> element
        if let Some(image) = element.try_into_type::<dom_objects::HtmlImageElement>() {
            let Some(texture) = image.borrow_mut().texture().cloned() else {
                // Fallback to an empty image with no intrinsic size
                let replaced_element = ReplacedElement {
                    intrinsic_size: IntrinsicSize::NONE,
                    content: ReplacedContent::Image(Texture::empty()),
                    style: element_style,
                };

                return Some(replaced_element);
            };

            let intrinsic_width = Pixels(texture.width() as f32);
            let intrinsic_height = Pixels(texture.height() as f32);

            let intrinsic_size = IntrinsicSize::new(intrinsic_width, intrinsic_height);
            let replaced_image = ReplacedElement {
                intrinsic_size,
                content: ReplacedContent::Image(texture),
                style: element_style,
            };
            Some(replaced_image)
        } else {
            None
        }
    }
}

impl ReplacedContent {
    /// Create a fragment for the given position and size
    ///
    /// This is where CSS hands over control to the replaced content, anything inside
    /// this fragment is not affected by the outside world anymore.
    #[must_use]
    pub fn create_fragment(&self, position: Vec2D<Pixels>, size: Size<Pixels>) -> Fragment {
        // FIXME: This is just a placeholder until we can dynamically load images from the "<img src=" attribute
        match self {
            Self::Image(texture) => TextureFragment {
                texture: texture.clone(),
                area: Rectangle::from_position_and_size(position, size.width, size.height),
            }
            .into(),
        }
    }
}
