#[deny(clippy::all)]
pub mod color;
pub mod events;
pub mod layout;
pub mod primitives;
pub mod surface;

extern crate sdl2;

use anyhow::Result;

// use std::time::Duration;

// use events::{
//     Event,
//     keyboard::KeyCode,
// };

// use font::ttf::Font;
// use layout::{
//     Sizing,
//     widgets::{ColoredBox, Input},
//     Divider, Orientation, Widget,
// };
// use surface::Surface;

pub fn main() -> Result<()> {
    // let response = Request::get("http://google.com/".into())?.send()?;
    // println!("{:?}", response.headers);
    // println!("{:?}", String::from_utf8_lossy(&response.body));
    if true {
        println!("test");
    }
    image::png::load_from_file("/home/alaska/Pictures/waifu.png")?;
    Ok(())
    // let font = Font::default();
    // println!("width: {}", font.compute_width("abc"));

    // let sdl_context = sdl2::init().unwrap();
    // let video_subsystem = sdl_context.video().unwrap();

    // let window = video_subsystem
    //     .window("Browser", 800, 600)
    //     .position_centered()
    //     // .borderless()
    //     .resizable()
    //     .build()
    //     .unwrap();

    // let mut canvas = Box::new(window.into_canvas().build().unwrap()) as Box<dyn Surface>;

    // canvas.fill(color::WHITE);

    // let mut textbox = Input::new(color::RED).as_widget();
    // textbox.set_size(Sizing::Exactly(50));

    // let mut root = Divider::new(Orientation::Vertical)
    //     .add_child(textbox)
    //     .add_child(ColoredBox::new(color::BLUE).as_widget());

    //     root.render(&mut canvas);
    // canvas.update();

    // let mut event_pump = sdl_context.event_pump().unwrap();
    // 'running: loop {
    //     for event in event_pump
    //         .poll_iter()
    //         .map(|sdlevent| Event::try_from(sdlevent))
    //         .filter(|event| event.is_ok())
    //         .map(|event| event.unwrap())
    //     {
    //         println!("{event:?}");
    //         match event {
    //             Event::Quit { .. } | Event::KeyDown { keycode: KeyCode::Escape, .. }=> break 'running,
    //             Event::Resize => {
    //                 root.invalidate_layout();
    //                 root.render(&mut canvas);
    //                 canvas.update();
    //             }
    //             _ => {
    //                 root.swallow_event(event);
    //             },
    //         }
    //     }

    //     ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    // }
}
