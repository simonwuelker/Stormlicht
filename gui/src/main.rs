use gtk::prelude::*;

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title("Iguana Browser")
        .default_width(800)
        .default_height(600)
        .build();

    let canvas = gtk::DrawingArea::new();
    canvas.set_draw_func(move |area, context, width, height| {
        context.set_source_rgb(1., 1., 1.);
        context.paint().expect("Painting failed");
    });

    window.set_child(Some(&canvas));
    window.show();
}

fn main() {
    let app = gtk::Application::builder()
        .application_id("com.github.wuelle.iguana")
        .build();

    app.connect_activate(build_ui);
    app.run();
}
