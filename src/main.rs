#![no_std]
#![no_main]

use defmt_rtt as _;
use hal::pac;

use embedded_hal::adc::OneShot;
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::PwmPin;

#[cfg(target_arch = "arm")]
use panic_probe as _;

// Alias for our HAL crate
use hal::entry;

#[cfg(rp2350)]
use rp235x_hal as hal;

// use bsp::entry;
// use bsp::hal;
// use rp_pico as bsp;

/// The linker will place this boot block at the start of our program image. We
/// need this to help the ROM bootloader get our code up and running.
/// Note: This boot block is not necessary when using a rp-hal based BSP
/// as the BSPs already perform this step.

/// Tell the Boot ROM about our application
#[unsafe(link_section = ".start_block")]
#[used]
#[cfg(rp2350)]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

/// External high-speed crystal on the Raspberry Pi Pico 2 board is 12 MHz.
/// Adjust if your board has a different frequency
const XTAL_FREQ_HZ: u32 = 12_000_000u32;
const DELAY: u32 = 1000; // loop delay
const SAMPLING_INTERVAL: u32 = 10;
const SAMPLES: u32 = 500;

/// Entry point to our bare-metal application.
///
/// The `#[hal::entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables and the spinlock are initialised.
///
/// The function configures the rp2040 and rp235x peripherals, then toggles a GPIO pin in
/// an infinite loop. If there is an LED connected to that pin, it will blink.
#[entry]
fn main() -> ! {
    // 1. Peripherals取得
    let mut pac = pac::Peripherals::take().unwrap();

    // 2. クロック・ウォッチドッグ初期化
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // 3. GPIO初期化
    let sio = hal::Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut delay = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);

    // 4. ADC設定
    let mut adc = hal::Adc::new(pac.ADC, &mut pac.RESETS);
    let mut adc_pin = hal::adc::AdcPin::new(pins.gpio26).unwrap();

    // 5. PWM設定
    let pwm_slices = hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS);
    let mut pwm = pwm_slices.pwm0;
    pwm.set_ph_correct();
    pwm.set_top(4095);
    pwm.enable();

    // GP0 = PWM Slice 0 Channel A(alomost 20KHz duty 99% @3.3v full scale)

    let mut pwm_channel = pwm.channel_a;
    let _pwm_pin = pins.gpio0.into_function::<hal::gpio::FunctionPwm>(); // GP0 as a PWM output

    loop {
        let mut sum: u32 = 0;

        // measure "samples" time and make average
        for _ in 0..SAMPLES {
            let v: u16 = adc.read(&mut adc_pin).unwrap();
            sum += v as u32;
            delay.delay_us(SAMPLING_INTERVAL);
        }

        let avg = (sum / SAMPLES) as u16;
        pwm_channel.set_duty(avg);
        delay.delay_us(DELAY);
    }
}

/// Program metadata for `picotool info`
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [hal::binary_info::EntryAddr; 5] = [
    hal::binary_info::rp_cargo_bin_name!(),
    hal::binary_info::rp_cargo_version!(),
    hal::binary_info::rp_program_description!(c"Blinky Example"),
    hal::binary_info::rp_cargo_homepage_url!(),
    hal::binary_info::rp_program_build_attribute!(),
];
