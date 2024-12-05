# pimoroni_gfx_pack
 
This rust library offers convenient functions for interacting with [Pico GFX Pack](https://shop.pimoroni.com/products/pico-gfx-pack?variant=40414469062739) - The Pico GFX Pack adds a 128x64 LCD Matrix display to your headered Raspberry Pi Pico or PicoW, with RGBW backlight and 5 input buttons.

#### This driver enables easy initialization and usage of the entire GFX Pack (including pico) with a single line of code:

```let mut gfx = GFXPACK::new();```

#### It also allows easily access gfx pack properties like display, buttons, rgbled:

```
//accessing display from GFXPACK and turning off backlight
gfx.display.backlight(BacklightStatus::Off).unwrap();

//accessing rgbled and setting color 
gfx.rgb.set_rgb_color(155, 93, 229);

//accessing button a and checking if pressed
gfx.button_a.is_button_pressed()
```


#### Example program - simple counter (press a button to increment , press b button to reset):

```
#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use itoa::Buffer;
use panic_probe as _;
//reexports rp2040_hal
use pimoroni_gfx_pack::hal::entry;
use pimoroni_gfx_pack::{BacklightStatus, ST7567Type, GFXPACK};

fn draw_text(s: &str, x: i32, y: i32, display: &mut ST7567Type) {
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    Text::new(s, Point::new(x, y), style).draw(display).unwrap();
}

#[entry]
fn main() -> ! {
    info!("Program start");
    //init whole gfx pack and rp2040 pico
    let mut gfx = GFXPACK::new();
    let mut counter = 0;
    //accessing display from GFXPACK and turning off backlight
    gfx.display.backlight(BacklightStatus::Off).unwrap();

    loop {
        //accessing rgbled and setting color
        gfx.rgb.set_rgb_color(155, 93, 229);
        //accessing button a and checking if pressed
        if gfx.button_a.is_button_pressed() {
            //clearing display
            gfx.display.clear().unwrap();
            counter += 1;
        } else if gfx.button_b.is_button_pressed() {
            gfx.display.clear().unwrap();
            counter = 0;
        }
        let mut buffer = Buffer::new();
        let counter_str = buffer.format(counter);
        let text = "Count: ";
        draw_text(text, 3, 45, &mut gfx.display);
        draw_text(counter_str, 58, 45, &mut gfx.display);
        gfx.display.show().unwrap();
    }
}
// End of file

```
