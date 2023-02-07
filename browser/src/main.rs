mod browser_application;

use anyhow::{anyhow, Result};
use browser_application::BrowserApplication;

use widgets::application::Application;

// fn map_image_format(format: PixelFormat) -> PixelFormatEnum {
//     match format {
//         PixelFormat::RGB8 => PixelFormatEnum::RGB24,
//         PixelFormat::RGBA8 => PixelFormatEnum::RGBA32,
//         _ => todo!("Find mapping for {format:?}"),
//     }
// }

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

    let mut application = BrowserApplication::default();
    application.run()
}
