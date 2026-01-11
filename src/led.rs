use core::ops::Fn;
use core::convert::From;
use arduino_hal::port::{Pin, PinOps};
use arduino_hal::port::mode::{Floating, Input, Output};

pub enum Color {
    RGB(u8, u8, u8),
    NUM(u32),
    HEX(&'static str),

    Black,
    Gray,
    White,
    Red,
    Green,
    Blue,
    Cyan,
    Magenta,
    Yellow,
    Orange,
    Purple,

    Pink,
    Turquoise,
}

impl Color {
    pub fn to_rgb(self) -> (u8, u8, u8) {
        match self {
            Color::RGB(red, green, blue) => (red, green, blue),
            Color::NUM(color) => (
                ((color << 8) & 0xFF) as u8,
                (color & 0xFF) as u8,
                ((color << 16) & 0xFF) as u8,
            ),
            Color::HEX(color) => {
                let bytes = parse(color);

                (bytes[0], bytes[1], bytes[2])
            },

            Color::Black => (0, 0, 0),
            Color::Gray => (127, 127, 127),
            Color::White => (255, 255, 255),
            Color::Red => (255, 0, 0),
            Color::Green => (0, 255, 0),
            Color::Blue => (0, 0, 255),
            Color::Cyan => (0, 255, 255),
            Color::Magenta => (255, 0, 255),
            Color::Yellow => (255, 255, 0),
            Color::Orange => (255, 127, 0),
            Color::Purple => (127, 0, 255),

            Color::Pink => (255, 127, 255),
            Color::Turquoise => (0, 127, 255),
        }
    }

    pub fn opacity(self, opacity: f64) -> Color {
        let (red, green, blue) = self.to_rgb();

        Color::RGB(
            (f64::from(red) * opacity).clamp(0.0, 255.0) as u8,
            (f64::from(green) * opacity).clamp(0.0, 255.0) as u8,
            (f64::from(blue) * opacity).clamp(0.0, 255.0) as u8,
        )
    }

    pub fn mix(self, color: Color) -> Color {
        self.mix_ratio(color, 0.5, 0.5)
    }

    pub fn mix_ratio(self, color: Color, self_ratio: f64, other_ratio: f64) -> Color {
        let (red, green, blue) = self.to_rgb();
        let (other_red, other_green, other_blue) = color.to_rgb();

        Color::RGB(
            (f64::from(red) * self_ratio + f64::from(other_red) * other_ratio).clamp(0.0, 255.0) as u8,
            (f64::from(green) * self_ratio + f64::from(other_green) * other_ratio).clamp(0.0, 255.0) as u8,
            (f64::from(blue) * self_ratio + f64::from(other_blue) * other_ratio).clamp(0.0, 255.0) as u8,
        )
    }
}

pub struct LedStrip<PIN: PinOps> {
    led_count: usize,
    pin: Pin<Output, PIN>,
}

impl<PIN> LedStrip<PIN> where PIN: PinOps {
    #[allow(dead_code)]
    pub fn new(led_count: usize, pin: Pin<Input<Floating>, PIN>) -> LedStrip<PIN> {
        LedStrip {
            led_count,
            pin: pin.into_output(),
        }
    }

    pub fn each<F>(&mut self, callback: F) where F: (Fn(usize) -> Color) {
        for led_index in 0..self.led_count {
            self.color(callback(led_index));
        }
    }

    pub fn hex(&mut self, color: &str) {
        let bytes = parse(color);
        self.rgb(bytes[0], bytes[1], bytes[2]);
    }

    pub fn rgb(&mut self, red: u8, green: u8, blue: u8) {
        send_byte(&mut self.pin, green);
        send_byte(&mut self.pin, red);
        send_byte(&mut self.pin, blue);
    }

    pub fn color(&mut self, color: Color) {
        let (red, green, blue) = color.to_rgb();

        self.rgb(red, green, blue);
    }

    pub fn color_number(&mut self, color: u32) {
        self.rgb(
            ((color << 8) & 0xFF) as u8,
            (color & 0xFF) as u8,
            ((color << 16) & 0xFF) as u8,
        );
    }

    pub fn rest(&mut self, us: u32) {
        reset(&mut self.pin, us);
    }
}

pub fn send_color<PIN: PinOps>(led: &mut Pin<Output, PIN>, color: &[u8]) -> () {
    let mut bytes = [0; 3];

    bytes.copy_from_slice(color);

    for byte in bytes {
        send_byte(led, byte);
    }
}

pub fn send_bit<PIN: PinOps>(led: &mut Pin<Output, PIN>, bit: u8) -> () {
    match bit {
        0 => zero(led),
        _ => one(led),
    };
}

pub fn send_byte<PIN: PinOps>(led: &mut Pin<Output, PIN>, byte: u8) -> () {
    for i in 0..8 {
        send_bit(led, byte & (1 << (7 - i)));
    }
}

pub fn reset<PIN: PinOps>(led: &mut Pin<Output, PIN>, us: u32) -> () {
    set_low_for(led, us * 1_000);
}

pub fn zero<PIN: PinOps>(led: &mut Pin<Output, PIN>) -> () {
    led.set_high();
    led.set_low();
    led.set_low();
    led.set_low();
    led.set_low();
    led.set_low();
}

pub fn one<PIN: PinOps>(led: &mut Pin<Output, PIN>) -> () {
    led.set_high();
    led.set_high();
    led.set_high();
    led.set_low();
    led.set_low();
    led.set_low();
}

pub fn set_high_for<PIN: PinOps>(led: &mut Pin<Output, PIN>, ns: u32) -> () {
    led.set_high();
    arduino_hal::delay_ns(ns);
}

pub fn set_low_for<PIN: PinOps>(led: &mut Pin<Output, PIN>, ns: u32) -> () {
    led.set_low();
    arduino_hal::delay_ns(ns);
}

pub fn parse(color: &str) -> [u8; 3] {
    u8::from_str_radix(&color[..2], 16)
        .and_then(|r| {
            u8::from_str_radix(&color[2..4], 16).and_then(
                |g| u8::from_str_radix(&color[4..6], 16).map(|b| [r, g, b]),
            )
        })
        .unwrap()
}

#[cfg(test)]
mod tests {
    use crate::led::Color;

    #[test]
    fn opacity() {
        assert_eq!(
            Color::Orange.opacity(0.5).to_rgb(),
            (127, 63, 0),
        );
        assert_eq!(
            Color::Orange.opacity(0.25).to_rgb(),
            (63, 31, 0),
        );
    }

    #[test]
    fn mix() {
        assert_eq!(
            Color::Red.mix(Color::Green).to_rgb(),
            Color::Yellow.opacity(0.5).to_rgb(),
        );
        assert_eq!(
            Color::Red.mix_ratio(Color::Green, 1.0, 1.0).to_rgb(),
            Color::Yellow.to_rgb(),
        );
        assert_eq!(
            Color::Red.mix_ratio(Color::Green, 0.5, 1.0).to_rgb(),
            (127, 255, 0),
        );
    }
}
