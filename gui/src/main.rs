mod pixelbuffer;
mod quad_bezier;

use font_rasterizer::ttf;

use pixelbuffer::{PixelBuffer, RendererTargetView, RendererTarget};

use gtk::cairo;
use gtk::prelude::*;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title("Iguana Browser")
        .default_width(WIDTH)
        .default_height(HEIGHT)
        .build();

    let canvas = gtk::DrawingArea::new();

    canvas.set_draw_func(move |_area, context, viewport_width, viewport_height| {
        let format = cairo::Format::Rgb24;
        let stride = format.stride_for_width(viewport_width as u32).unwrap();
        let buffer_size = (stride * viewport_height * 4) as usize;

        let mut data: Vec<u8> = Vec::with_capacity(buffer_size);
        data.resize(buffer_size, 0);
        let mut pixelbuffer = PixelBuffer::new(
            &mut data,
            viewport_width as usize,
            viewport_height as usize,
            stride as usize,
        );

        pixelbuffer.fill((0, 255, 0));
        pixelbuffer.line((10, 10), (300, 100), (255, 0, 0));

        let mut view = RendererTargetView::new(pixelbuffer, (100, 100), 50, 100);
        view.fill((0, 0, 255));
        let pixelbuffer = view.release();

        let surface = cairo::ImageSurface::create_for_data(
            data,
            format,
            viewport_width,
            viewport_height,
            stride,
        )
        .unwrap();

        context.set_source_surface(surface, 0., 0.).unwrap();
        context.paint().expect("Painting failed");
    });

    window.set_child(Some(&canvas));
    window.show();
}

fn main() {
    // Parse OpenSans font
    let font_bytes = include_bytes!("../Envy Code R.ttf");
    ttf::parse_font_face(font_bytes).unwrap();

    let app = gtk::Application::builder()
        .application_id("com.github.wuelle.iguana")
        .build();

    app.connect_activate(build_ui);
    app.run();
}
