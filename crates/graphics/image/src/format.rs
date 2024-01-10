use std::convert::FloatToInt;

use crate::Texture;

macro_rules! color_format {
    ($name: ident, $n_channels: expr, $identifier: literal) => {
        #[derive(Clone, Copy, Debug)]
        #[repr(transparent)]
        pub struct $name<T>([T; $n_channels]);

        impl<T> ColorFormat for $name<T>
        where
            T: ColorChannel,
            f32: FloatToInt<T>,
        {
            type Channel = T;

            const NAME: &'static str = $identifier;
            const N_CHANNELS: usize = $n_channels;

            #[must_use]
            fn from_channels(channels: &[T]) -> Self {
                Self(channels.try_into().expect("Incorrect number of channels"))
            }

            #[must_use]
            fn channels(&self) -> &[T] {
                self.0.as_slice()
            }

            #[must_use]
            fn channels_mut(&mut self) -> &mut [T] {
                self.0.as_mut_slice()
            }

            #[must_use]
            fn blend(&self, other: Self) -> Self {
                Blend::blend(self, other)
            }
        }
    };
}

color_format!(Rgb, 3, "RGB");
color_format!(Rgba, 4, "RGBA");
color_format!(GrayScale, 1, "GrayScale");
color_format!(GrayScaleAlpha, 2, "GrayScaleAlpha");

impl<T> Default for Rgb<T>
where
    T: ColorChannel,
    f32: FloatToInt<T>,
{
    fn default() -> Self {
        Self([T::MIN, T::MIN, T::MIN])
    }
}

impl<T> Default for Rgba<T>
where
    T: ColorChannel,
    f32: FloatToInt<T>,
{
    fn default() -> Self {
        Self([T::MIN, T::MIN, T::MIN, T::MAX])
    }
}

impl<T> Default for GrayScale<T>
where
    T: ColorChannel,
    f32: FloatToInt<T>,
{
    fn default() -> Self {
        Self([T::MIN])
    }
}

impl<T> Default for GrayScaleAlpha<T>
where
    T: ColorChannel,
    f32: FloatToInt<T>,
{
    fn default() -> Self {
        Self([T::MIN, T::MAX])
    }
}

pub trait ColorChannel: Copy + Default + Into<f32> + PartialEq + Eq
where
    f32: FloatToInt<Self>,
{
    const MIN: Self;
    const MAX: Self;
}

impl ColorChannel for u8 {
    const MIN: Self = Self::MIN;
    const MAX: Self = Self::MAX;
}

pub trait ColorFormat: Copy + Default
where
    f32: FloatToInt<Self::Channel>,
{
    type Channel: ColorChannel;

    const NAME: &'static str;
    const N_CHANNELS: usize;

    /// Convert from a list of channels to the color value
    ///
    /// This function should panic if a number of channels other than `N_CHANNELS` is passed.
    fn from_channels(channels: &[Self::Channel]) -> Self;
    fn channels(&self) -> &[Self::Channel];
    fn channels_mut(&mut self) -> &mut [Self::Channel];
    fn blend(&self, other: Self) -> Self;
}

/// Blend two colors together
///
/// Extracted into its own trait so we can use macros for the [ColorFormat] implementation.
/// Don't use this trait directly.
trait Blend {
    /// Blend `other` (the foreground) onto `self` (the background)
    fn blend(&self, other: Self) -> Self;
}

impl<T> Blend for Rgb<T>
where
    T: ColorChannel,
    f32: FloatToInt<T>,
{
    fn blend(&self, other: Self) -> Self {
        other
    }
}

impl<T> Blend for Rgba<T>
where
    T: ColorChannel,
    f32: FloatToInt<T>,
{
    fn blend(&self, other: Self) -> Self {
        // https://stackoverflow.com/questions/7438263/alpha-compositing-algorithm-blend-modes#answer-11163848
        let foreground_channels = other.channels();
        let background_channels = self.channels();

        if foreground_channels[3] == T::MIN {
            return *self;
        }

        if foreground_channels[3] == T::MAX {
            return other;
        }

        // Map all values into range [0, 1]
        let max_range: f32 = T::MAX.into();
        let foreground_channels = &[
            foreground_channels[0].into() / max_range,
            foreground_channels[1].into() / max_range,
            foreground_channels[2].into() / max_range,
            foreground_channels[3].into() / max_range,
        ];
        let background_channels = &[
            background_channels[0].into() / max_range,
            background_channels[1].into() / max_range,
            background_channels[2].into() / max_range,
            background_channels[3].into() / max_range,
        ];

        let foreground_alpha = foreground_channels[3];
        let background_alpha = background_channels[3];

        let alpha = background_alpha + foreground_alpha - background_alpha * foreground_alpha;

        if alpha == 0. {
            return Self([T::MIN, T::MIN, T::MIN, T::MIN]);
        }

        let foreground_channels = &[
            foreground_channels[0] * foreground_alpha,
            foreground_channels[1] * foreground_alpha,
            foreground_channels[2] * foreground_alpha,
        ];
        let background_channels = &[
            background_channels[0] * background_alpha,
            background_channels[1] * background_alpha,
            background_channels[2] * background_alpha,
        ];

        let red = foreground_channels[0] + background_channels[0] * (1. - foreground_alpha);
        let green = foreground_channels[1] + background_channels[1] * (1. - foreground_alpha);
        let blue = foreground_channels[2] + background_channels[2] * (1. - foreground_alpha);

        // SAFETY: any color / alpha is guaranteed to be finite (since alpha is not zero, and the color is not infinite)
        //         and is guaranteed to fit in the resulting type due to the nature of this formula
        unsafe {
            Self([
                (red / alpha).to_int_unchecked(),
                (green / alpha).to_int_unchecked(),
                (blue / alpha).to_int_unchecked(),
                alpha.to_int_unchecked(),
            ])
        }
    }
}

impl<T> Blend for GrayScale<T>
where
    T: ColorChannel,
    f32: FloatToInt<T>,
{
    fn blend(&self, other: Self) -> Self {
        other
    }
}

impl<T> Blend for GrayScaleAlpha<T>
where
    T: ColorChannel,
    f32: FloatToInt<T>,
{
    fn blend(&self, other: Self) -> Self {
        // https://stackoverflow.com/questions/7438263/alpha-compositing-algorithm-blend-modes#answer-11163848
        let &[foreground_luminosity, foreground_alpha] = other.channels() else {
            unreachable!("expected exactly two channels")
        };
        let &[background_luminosity, background_alpha] = self.channels() else {
            unreachable!("expected exactly two channels")
        };

        if foreground_alpha == T::MIN {
            return *self;
        }

        if foreground_alpha == T::MAX {
            return other;
        }

        // Map all values into range [0, 1]
        let max_range: f32 = T::MAX.into();
        let foreground_luminosity = foreground_luminosity.into() / max_range;
        let foreground_alpha = foreground_alpha.into() / max_range;
        let background_luminosity = background_luminosity.into() / max_range;
        let background_alpha = background_alpha.into() / max_range;

        let alpha = background_alpha + foreground_alpha - background_alpha * foreground_alpha;

        if alpha == 0. {
            return Self([T::MIN, T::MIN]);
        }

        let foreground_luminosity = foreground_luminosity * foreground_alpha;
        let background_luminosity = background_luminosity * background_alpha;

        let value = foreground_luminosity + background_luminosity * (1. - foreground_alpha);

        // SAFETY: any color / alpha is guaranteed to be finite (since alpha is not zero, and the color is not infinite)
        //         and is guaranteed to fit in the resulting type due to the nature of this formula
        unsafe { Self([(value / alpha).to_int_unchecked(), alpha.to_int_unchecked()]) }
    }
}

impl<T> From<Rgba<T>> for Rgb<T>
where
    T: ColorChannel,
    f32: FloatToInt<T>,
{
    fn from(value: Rgba<T>) -> Self {
        let channels = value.channels();
        Self([channels[0], channels[1], channels[2]])
    }
}

impl<T> From<GrayScale<T>> for Rgb<T>
where
    T: ColorChannel,
    f32: FloatToInt<T>,
{
    fn from(value: GrayScale<T>) -> Self {
        let channels = value.channels();
        Self([channels[0], channels[0], channels[0]])
    }
}

impl<T> From<GrayScaleAlpha<T>> for Rgb<T>
where
    T: ColorChannel,
    f32: FloatToInt<T>,
{
    fn from(value: GrayScaleAlpha<T>) -> Self {
        let channels = value.channels();
        Self([channels[0], channels[0], channels[0]])
    }
}

impl<T> From<Rgb<T>> for Rgba<T>
where
    T: ColorChannel,
    f32: FloatToInt<T>,
{
    fn from(value: Rgb<T>) -> Self {
        let channels = value.channels();
        Self([channels[0], channels[1], channels[2], T::MAX])
    }
}

impl<T> From<GrayScale<T>> for Rgba<T>
where
    T: ColorChannel,
    f32: FloatToInt<T>,
{
    fn from(value: GrayScale<T>) -> Self {
        let channels = value.channels();
        Self([channels[0], channels[0], channels[0], T::MAX])
    }
}

impl<T> From<GrayScaleAlpha<T>> for Rgba<T>
where
    T: ColorChannel,
    f32: FloatToInt<T>,
{
    fn from(value: GrayScaleAlpha<T>) -> Self {
        let channels = value.channels();
        Self([channels[0], channels[0], channels[0], channels[1]])
    }
}

#[derive(Clone, Debug)]
pub enum DynamicTexture {
    GrayScale8(Texture<GrayScale<u8>, Vec<u8>>),
    GrayScaleAlpha8(Texture<GrayScaleAlpha<u8>, Vec<u8>>),
    Rgb8(Texture<Rgb<u8>, Vec<u8>>),
    Rgba8(Texture<Rgba<u8>, Vec<u8>>),
}

impl From<Texture<GrayScale<u8>, Vec<u8>>> for DynamicTexture {
    fn from(value: Texture<GrayScale<u8>, Vec<u8>>) -> Self {
        Self::GrayScale8(value)
    }
}

impl From<Texture<GrayScaleAlpha<u8>, Vec<u8>>> for DynamicTexture {
    fn from(value: Texture<GrayScaleAlpha<u8>, Vec<u8>>) -> Self {
        Self::GrayScaleAlpha8(value)
    }
}

impl From<Texture<Rgb<u8>, Vec<u8>>> for DynamicTexture {
    fn from(value: Texture<Rgb<u8>, Vec<u8>>) -> Self {
        Self::Rgb8(value)
    }
}

impl From<Texture<Rgba<u8>, Vec<u8>>> for DynamicTexture {
    fn from(value: Texture<Rgba<u8>, Vec<u8>>) -> Self {
        Self::Rgba8(value)
    }
}
