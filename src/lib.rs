#![no_std]
#![no_main]

use embedded_hal::spi::{Mode, Phase, Polarity};
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};

use pimoroni_gfx_pack_button::Button;
use rgbled::RGB;

pub extern crate rp2040_hal as hal;

#[cfg(feature = "rt")]
pub use hal::{entry, rosc};

#[cfg(feature = "rt")]
extern crate cortex_m_rt;

#[cfg(feature = "boot2")]
#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

use hal::{
    clocks::{init_clocks_and_plls, Clock},
    fugit::RateExtU32,
    gpio::{
        bank0::{Gpio18, Gpio19},
        DynPinId, FunctionSio, FunctionSpi, Pin, PinState, Pins, PullDown, PullUp, SioInput,
        SioOutput,
    },
    pac::{Peripherals, SPI0},
    pwm::{Channel, FreeRunning, Pwm3, Pwm4, Slice, Slices, A, B},
    rosc::{Disabled, RingOscillator},
    sio::Sio,
    spi::Enabled,
    watchdog::Watchdog,
    Spi, Timer,
};

pub use st7567_rs::BacklightStatus;
use st7567_rs::ST7567;

pub type ButtonType = Button<Pin<DynPinId, FunctionSio<SioInput>, PullUp>, Timer>;
pub type RGBType = RGB<
    Channel<Slice<Pwm3, FreeRunning>, A>,
    Channel<Slice<Pwm3, FreeRunning>, B>,
    Channel<Slice<Pwm4, FreeRunning>, A>,
>;
pub type ST7567Pin = Pin<DynPinId, FunctionSio<SioOutput>, PullDown>;

pub type ST7567SPIDevice = ExclusiveDevice<
    Spi<
        Enabled,
        SPI0,
        (
            Pin<Gpio19, FunctionSpi, PullDown>,
            Pin<Gpio18, FunctionSpi, PullDown>,
        ),
    >,
    ST7567Pin,
    NoDelay,
>;

pub type ST7567Type = ST7567<ST7567Pin, ST7567Pin, ST7567Pin, ST7567SPIDevice>;

pub struct GFXPACK {
    pub display: ST7567Type,
    pub button_a: ButtonType,
    pub button_b: ButtonType,
    pub button_c: ButtonType,
    pub button_d: ButtonType,
    pub button_e: ButtonType,
    pub rgb: RGBType,
    pub delay: Timer,
    pub rosc: RingOscillator<Disabled>,
}

impl GFXPACK {
    pub fn new() -> Self {
        let mut pac = Peripherals::take().unwrap();
        let mut watchdog = Watchdog::new(pac.WATCHDOG);
        let sio = Sio::new(pac.SIO);

        // External high-speed crystal on the pico board is 12Mhz
        let external_xtal_freq_hz = 12_000_000u32;
        let clocks = init_clocks_and_plls(
            external_xtal_freq_hz,
            pac.XOSC,
            pac.CLOCKS,
            pac.PLL_SYS,
            pac.PLL_USB,
            &mut pac.RESETS,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        let pins = Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );

        let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

        let _cs = pins
            .gpio17
            .into_push_pull_output_in_state(PinState::High)
            .into_dyn_pin();

        let _sck = pins.gpio18.into_function::<FunctionSpi>();
        let _mosi = pins.gpio19.into_function::<FunctionSpi>();

        let dcx = pins.gpio20.into_push_pull_output().into_dyn_pin();
        let blx = pins.gpio9.into_push_pull_output().into_dyn_pin();
        let rst = pins.gpio21.into_push_pull_output().into_dyn_pin();

        let mode = Mode {
            phase: Phase::CaptureOnSecondTransition,
            polarity: Polarity::IdleHigh,
        };

        let spi = Spi::<_, _, _, 8>::new(pac.SPI0, (_mosi, _sck));
        let spi = spi.init(
            &mut pac.RESETS,
            clocks.peripheral_clock.freq(),
            10_000_000u32.Hz(),
            mode,
        );

        //think how to get rid off. maybe modify st7567 driver to pass cs pin?
        let spi_device = ExclusiveDevice::new_no_delay(spi, _cs);

        let mut display = ST7567::new(dcx, blx, rst, spi_device);
        display.backlight(BacklightStatus::Off).unwrap(); //off in case of rgb on
        display.init().unwrap();

        let rosc = RingOscillator::new(pac.ROSC);
        let pwm_slices = Slices::new(pac.PWM, &mut pac.RESETS);

        // Configure PWM 3 or 4
        let mut pwm = pwm_slices.pwm3;
        pwm.set_ph_correct();
        pwm.enable();
        let mut pwm2 = pwm_slices.pwm4;
        pwm2.set_ph_correct();
        pwm2.enable();

        let mut r_pin_channel = pwm.channel_a;
        r_pin_channel.output_to(pins.gpio6);
        let mut g_pin_channel = pwm.channel_b;
        g_pin_channel.output_to(pins.gpio7);
        let mut b_pin_channel = pwm2.channel_a;
        b_pin_channel.output_to(pins.gpio8);

        //rgbled init
        let rgb = RGB::new(r_pin_channel, g_pin_channel, b_pin_channel);

        //buttons init
        let button_a = Button::new(pins.gpio12.into_pull_up_input().into_dyn_pin(), timer);
        let button_b = Button::new(pins.gpio13.into_pull_up_input().into_dyn_pin(), timer);
        let button_c = Button::new(pins.gpio14.into_pull_up_input().into_dyn_pin(), timer);
        let button_d = Button::new(pins.gpio15.into_pull_up_input().into_dyn_pin(), timer);
        let button_e = Button::new(pins.gpio22.into_pull_up_input().into_dyn_pin(), timer);

        Self {
            display,
            button_a,
            button_b,
            button_c,
            button_d,
            button_e,
            rgb,
            delay: timer,
            rosc,
        }
    }
}
