use math::Vec2D;
use render::{Composition, Path, Source};

use crate::css::{
    display_list::{
        command::{RectCommand, TextCommand},
        Command,
    },
    layout::{Pixels, Size},
    FontMetrics,
};

#[derive(Clone, Debug, Default)]
pub struct Painter {
    commands: Vec<Command>,
}

impl Painter {
    pub fn paint_magic_background(&mut self, viewport: Size<Pixels>, color: math::Color) {
        let area = viewport.at_position(math::Vec2D::new(Pixels::ZERO, Pixels::ZERO));

        self.commands
            .insert(0, Command::Rect(RectCommand { area, color }))
    }

    pub fn rect(&mut self, area: math::Rectangle<Pixels>, color: math::Color) {
        self.commands
            .push(Command::Rect(RectCommand { area, color }))
    }

    pub fn text(
        &mut self,
        text: String,
        position: Vec2D<Pixels>,
        color: math::Color,
        font_metrics: FontMetrics,
    ) {
        let text_command = TextCommand {
            position,
            text,
            font_metrics,
            color,
        };

        self.commands.push(Command::Text(text_command));
    }

    pub fn paint(&self, composition: &mut Composition) {
        for (index, command) in self.commands.iter().enumerate() {
            match command {
                Command::Rect(rect_cmd) => {
                    composition
                        .get_or_insert_layer(index as u16)
                        .with_source(Source::Solid(rect_cmd.color))
                        .with_outline(Path::rect(
                            Vec2D {
                                x: rect_cmd.area.top_left().x.0,
                                y: rect_cmd.area.top_left().y.0,
                            },
                            Vec2D {
                                x: rect_cmd.area.bottom_right().x.0,
                                y: rect_cmd.area.bottom_right().y.0,
                            },
                        ));
                },
                Command::Text(text_command) => {
                    composition
                        .get_or_insert_layer(index as u16)
                        .text(
                            &text_command.text,
                            *text_command.font_metrics.font_face.clone(),
                            text_command.font_metrics.size.into(),
                            Vec2D {
                                x: text_command.position.x.0,
                                y: text_command.position.y.0,
                            },
                        )
                        .with_source(Source::Solid(text_command.color));
                },
            }
        }
    }
}
