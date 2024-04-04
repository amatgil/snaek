use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(4))]
pub struct Pixel {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Pixel {
    #[inline]
    pub fn new(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self { a, r, g, b }
    }

    #[inline]
    pub fn from_hex(hex: u32) -> Self {
        // We're doing math here to not assume big- or little-endian

        Self {
            a: ((hex >> 24) & 0xff) as u8,
            r: ((hex >> 16) & 0xff) as u8,
            g: ((hex >> 8) & 0xff) as u8,
            b: (hex & 0xff) as u8,
        }
    }

    #[inline]
    pub fn to_u32(self) -> u32 {
        // We're doing math here to not assume big- or little-endian

        let mut val = (self.a as u32) << 24;
        val |= (self.r as u32) << 16;
        val |= (self.g as u32) << 8;
        val |= self.b as u32;
        val
    }

    #[inline]
    pub fn to_array(self) -> [u8; 4] {
        unsafe { std::mem::transmute(self) }
    }
}

impl Add for Pixel {
    type Output = Pixel;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.a += rhs.a;
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
        self
    }
}

impl AddAssign for Pixel {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Pixel {
    type Output = Pixel;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self.a -= rhs.a;
        self.r -= rhs.r;
        self.g -= rhs.g;
        self.b -= rhs.b;
        self
    }
}

impl SubAssign for Pixel {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul<u8> for Pixel {
    type Output = Pixel;

    fn mul(mut self, rhs: u8) -> Self::Output {
        self.a *= rhs;
        self.r *= rhs;
        self.g *= rhs;
        self.b *= rhs;
        self
    }
}

impl MulAssign<u8> for Pixel {
    fn mul_assign(&mut self, rhs: u8) {
        *self = *self * rhs;
    }
}

impl Div<u8> for Pixel {
    type Output = Pixel;

    fn div(mut self, rhs: u8) -> Self::Output {
        self.a /= rhs;
        self.r /= rhs;
        self.g /= rhs;
        self.b /= rhs;
        self
    }
}

impl DivAssign<u8> for Pixel {
    fn div_assign(&mut self, rhs: u8) {
        *self = *self / rhs;
    }
}

/// Takes the RGB of the first pixel and the A of the second pixel.
#[inline]
pub fn rgb_a(mut rgb: Pixel, a: Pixel) -> Pixel {
    rgb.a = a.a;
    rgb
}

pub mod alphacomp {
    //! Alpha composition functions.
    //!
    //! ![Alpha compositing](https://upload.wikimedia.org/wikipedia/commons/thumb/2/2a/Alpha_compositing.svg/642px-Alpha_compositing.svg.png)

    use super::Pixel;

    #[inline]
    pub fn over(pixa: Pixel, pixb: Pixel) -> Pixel {
        let a = (pixa.a as u32 + pixb.a as u32 * (255 - pixa.a as u32)) as u8;
        let r = ((pixa.r as u32 + pixb.r as u32 * (255 - pixa.a as u32)) / a as u32) as u8;
        let g = ((pixa.g as u32 + pixb.g as u32 * (255 - pixa.a as u32)) / a as u32) as u8;
        let b = ((pixa.b as u32 + pixb.b as u32 * (255 - pixa.a as u32)) / a as u32) as u8;
        Pixel { a, r, g, b }
    }

    #[inline]
    pub fn add(pixa: Pixel, pixb: Pixel) -> Pixel {
        Pixel {
            a: pixa.a.saturating_add(pixb.a),
            r: pixa.r.saturating_add(pixb.r),
            g: pixa.g.saturating_add(pixb.g),
            b: pixa.b.saturating_add(pixb.b),
        }
    }
}