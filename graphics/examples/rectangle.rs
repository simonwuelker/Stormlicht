use graphics::{AffineTransform, Color, Compositor, Path, Renderer, Vec2D};

fn main() {
    let mut compositor = Compositor::default();
    compositor
        .get_or_insert_layer(0)
        .set_color(Color::rgb(255, 111, 200))
        .set_transform(AffineTransform::scale(2., 1.))
        .add_path(
            Path::new(Vec2D::new(0., 0.))
                .line_to(Vec2D::new(1., 0.))
                .line_to(Vec2D::new(1., 1.))
                .line_to(Vec2D::new(0., 1.))
                .close_contour(),
        );

    Renderer::render(&mut compositor);
}
