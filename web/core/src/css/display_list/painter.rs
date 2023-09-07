use math::Vec2D;
use render::{Composition, Path, Source};

use super::Command;

#[derive(Clone, Debug, Default)]
pub struct Painter<'box_tree, 'font> {
    commands: Vec<Command<'box_tree, 'font>>,
}

impl<'box_tree, 'font> Painter<'box_tree, 'font> {
    pub fn add_cmd(&mut self, command: Command<'box_tree, 'font>) {
        self.commands.push(command)
    }

    pub fn paint(&self, composition: &mut Composition) {
        for (index, command) in self.commands.iter().enumerate() {
            match command {
                Command::Rect(rect_cmd) => {
                    let area = composition
                        .get_or_insert_layer(index as u16)
                        .with_source(Source::Solid(rect_cmd.color))
                        .with_outline(Path::rect(
                            Vec2D {
                                x: rect_cmd.area.top_left.x.0,
                                y: rect_cmd.area.top_left.y.0,
                            },
                            Vec2D {
                                x: rect_cmd.area.bottom_right.x.0,
                                y: rect_cmd.area.bottom_right.y.0,
                            },
                        ));
                },
                Command::Text(_text_command) => {
                    todo!()
                },
            }
        }
    }
}
