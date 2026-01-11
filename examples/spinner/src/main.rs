#![no_std]
#![cfg_attr(not(test), no_main)]

use arduino_hal;
use led_strip::led::{LedStrip};
use led_strip::led::Color;

const NUM_LEDS: usize = 12;
const RESET_TIME: u32 = 200_000; // microseconds
const FRAMES: u16 = NUM_LEDS as u16;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();

    let mut led_strip = LedStrip::new(NUM_LEDS, arduino_hal::pins!(dp).d4);
    let mut frame: u16 = 0;

    loop {
        let color_1 = Color::Green.mix_ratio(Color::Blue, 1.0, 0.3);
        let color_2 = color_1.opacity(1.0 / 6.0);
        let color_3 = color_1.opacity(1.0 / 12.0);
        let color_4 = color_1.opacity(1.0 / 24.0);

        led_strip.rest(RESET_TIME);
        led_strip.each(|led_index| {
            match (frame + led_index as u16) % NUM_LEDS as u16 {
                0 => color_1,
                1 => color_2,
                2 => color_3,
                3 => color_4,
                _ => Color::Black,
            }
        });

        frame = (frame + 1) % FRAMES;
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
