pub mod color;
pub mod events;
pub mod layout;
pub mod primitives;
pub mod surface;

extern crate sdl2;

use std::time::Duration;

use events::Event;
use layout::{
    widgets::{ColoredBox, Input},
    Divider, Orientation, Widget,
};
use surface::Surface;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Browser", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = Box::new(window.into_canvas().build().unwrap()) as Box<dyn Surface>;

    canvas.fill(color::WHITE);

    let mut layout = Divider::new(Orientation::Vertical, 0.5)
        .set_first(Some(Input::new(color::RED).as_widget()))
        .set_second(Some(ColoredBox::new(color::BLUE).as_widget()));

    layout.render(&mut canvas);
    canvas.update();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump
            .poll_iter()
            .map(|sdlevent| Event::try_from(sdlevent))
            .filter(|event| event.is_ok())
            .map(|event| event.unwrap())
        {
            println!("{event:?}");
            match event {
                Event::Quit { .. } => break 'running,
                _ => {
                    layout.swallow_event(event);
                },
            }
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
