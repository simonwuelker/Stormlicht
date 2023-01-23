use anyhow::{anyhow, Result};
use canvas::{Canvas, PixelFormat};
use font::Font;
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
    pixels::PixelFormatEnum,
};
use widgets::{
    // colorscheme,
    // layout::{
    //     widgets::{ColoredBox, Input},
    //     Divider, Orientation, Sizing, Widget,
    // },
    sdl2::{self},
    GuiError,
};

fn map_image_format(format: PixelFormat) -> PixelFormatEnum {
    match format {
        PixelFormat::RGB8 => PixelFormatEnum::RGB24,
        PixelFormat::RGBA8 => PixelFormatEnum::RGBA32,
        _ => todo!("Find mapping for {format:?}"),
    }
}

#[cfg(target_os = "linux")]
#[link(name = "c")]
extern "C" {
    fn geteuid() -> u32;
}

pub fn main() -> Result<()> {
    #[cfg(target_os = "linux")]
    if unsafe { geteuid() } == 0 {
        return Err(anyhow!("Refusing to run as root"));
    }

    // let response = Request::get("http://google.com/".into())?.send()?;
    // println!("{:?}", response.headers);
    // println!("{:?}", String::from_utf8_lossy(&response.body));
    // let image = image::png::load_from_file("/home/alaska/Pictures/waifu.png")?;
    const FONT_SIZE: usize = 40;
    let mut image = Canvas::new_uninit(300, 100, PixelFormat::RGB8);
    let font = Font::default();
    println!("units per em: {}", font.units_per_em());
    font.render_text("A", &mut image, &[255, 255, 255], FONT_SIZE);
    println!("width: {}", font.compute_width("A", FONT_SIZE));

    let sdl_context = sdl2::init().map_err(GuiError::from_sdl)?;
    let video_subsystem = sdl_context.video().map_err(GuiError::from_sdl)?;

    let window = video_subsystem
        .window("Browser", image.width() as u32, image.height() as u32)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build()?;

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_target(
        Some(map_image_format(image.format())),
        image.width() as u32,
        image.height() as u32,
    )?;

    texture.update(
        None,
        image.data(),
        image.width() * image.format().pixel_size(),
    )?;
    canvas
        .copy(&texture, None, None)
        .map_err(GuiError::from_sdl)?;
    canvas.present();

    // let mut textbox = Input::new(colorscheme::BACKGROUND_DARK).into_widget();
    // textbox.set_size(Sizing::Exactly(50));

    // let mut root = Divider::new(Orientation::Vertical)
    //     .add_child(textbox)
    //     .add_child(ColoredBox::new(colorscheme::BACKGROUND_LIGHT).into_widget());

    // root.render(&mut canvas)?;
    // canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::Window {
                    win_event: WindowEvent::Resized(_width, _height),
                    ..
                } => {
                    // root.invalidate_layout();
                    // root.render(&mut canvas)?;
                    // canvas.present();
                },
                _ => {
                    // root.swallow_event(event);
                },
            }
        }
        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
    Ok(())
}
