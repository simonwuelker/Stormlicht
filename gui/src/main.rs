mod pixelbuffer;

#[cfg(feature = "backend-wayland")]
mod wayland;

// use font_rasterizer::{target::BoundingBox, ttf};

use pixelbuffer::{PixelBuffer, RendererTarget};

#[cfg(feature = "backend-gtk")]
use gtk::{
    cairo,
    prelude::*,
};

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;

// fn build_ui(application: &gtk::Application) {
//     let window = gtk::ApplicationWindow::builder()
//         .application(application)
//         .title("Iguana Browser")
//         .default_width(WIDTH)
//         .default_height(HEIGHT)
//         .build();
// 
//     let canvas = gtk::DrawingArea::new();
// 
//     canvas.set_draw_func(move |_area, context, viewport_width, viewport_height| {
//         let format = cairo::Format::Rgb24;
//         let stride = format.stride_for_width(viewport_width as u32).unwrap();
//         let buffer_size = (stride * viewport_height * 4) as usize;
// 
//         let mut data: Vec<u8> = Vec::with_capacity(buffer_size);
//         data.resize(buffer_size, 0);
//         let mut pixelbuffer = PixelBuffer::new(
//             &mut data,
//             viewport_width as usize,
//             viewport_height as usize,
//             stride as usize,
//         );
// 
//         pixelbuffer.fill((255, 255, 255));
// 
//         let font_bytes = include_bytes!("../Envy Code R.ttf");
//         let font = ttf::Font::new(font_bytes.as_slice()).unwrap();
// 
//         let text = "Hello W";
// 
//         let mut x = 0;
//         for c in text.chars() {
//             let a_glyph = font.get_glyph(c as u16).unwrap();
//             a_glyph.rasterize(&mut pixelbuffer, BoundingBox::new(10 + x, 10, 60 + x, 60));
//             x += 70;
//         }
// 
//         let surface = cairo::ImageSurface::create_for_data(
//             data,
//             format,
//             viewport_width,
//             viewport_height,
//             stride,
//         )
//         .unwrap();
// 
//         context.set_source_surface(surface, 0., 0.).unwrap();
//         context.paint().expect("Painting failed");
//     });
// 
//     window.set_child(Some(&canvas));
//     window.show();
// }

fn main() {
    wayland::try_init().unwrap();
    // let app = gtk::Application::builder()
    //     .application_id("com.github.wuelle.iguana")
    //     .build();

    // app.connect_activate(build_ui);
    // app.run();
}
