use dom_derive::inherit;
use image::Texture;
use resourceloader::RESOURCE_LOADER;
use url::URL;

use crate::static_interned;

use super::HtmlElement;

/// <https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element>
#[inherit(HtmlElement)]
pub struct HtmlImageElement {
    texture: Option<Option<Texture>>,
}

impl HtmlImageElement {
    pub fn new(html_element: HtmlElement) -> Self {
        // We can't load the image data here because the "src" attribute is only
        // assigned *after* calling this method
        Self {
            __parent: html_element,
            texture: None,
        }
    }

    #[must_use]
    pub fn texture(&mut self) -> Option<&Texture> {
        let loaded_texture = self
            .texture
            .get_or_insert_with(|| load_texture_for_img_element(&self.__parent));

        loaded_texture.as_ref()
    }
}

#[must_use]
fn load_texture_for_img_element(html_element: &HtmlElement) -> Option<Texture> {
    let Some(source_url) = html_element.attributes().get(&static_interned!("src")) else {
        log::error!("Failed to load <img> content: No \"src\" attribute found");
        return None;
    };

    let source_url = source_url.to_string();

    let source_url: URL = source_url.parse()
        .inspect_err(|error| {
            log::error!("Failed to load <img> content: \"src\" attribute ({source_url}) cannot be parsed as a URL ({error:?}")
        })
        .ok()?;

    let resource_handle = RESOURCE_LOADER.schedule_load(source_url.clone()).block();

    let resource = resource_handle
        .inspect_err(|error| {
            log::error!("Failed to load <img> content: {source_url} could not be loaded ({error:?}")
        })
        .ok()?;

    if !resource.metadata.computed_mime_type.is_image() {
        log::error!(
            "Failed to load <img> content: Expected image, found {}",
            resource.metadata.computed_mime_type
        );
        return None;
    }

    let texture = Texture::from_bytes(&resource.data)
        .inspect_err(|error| {
            log::error!(
                "Failed to load <img> content: Failed to load {source_url} as an image ({error:?})",
            )
        })
        .ok()?;

    Some(texture)
}
