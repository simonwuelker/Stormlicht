use graphics::{math::Vec2D, Color, Compositor, Path, Renderer};

fn main() {
    let mut compositor = Compositor::default();
    compositor
        .get_or_insert_layer(0)
        .set_color(Color::rgb(255, 111, 200))
        .scale(2., 1.)
        .add_path(
            Path::new(Vec2D::new(0., 0.))
                .line_to(Vec2D::new(5., 0.))
                .line_to(Vec2D::new(5., 5.))
                .line_to(Vec2D::new(0., 5.))
                .close_contour(),
        );

    Renderer::render(&mut compositor, 10, 10);
}
