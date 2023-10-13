#![no_std]
#![no_main]

use core::cell::RefCell;
use core::num::{NonZeroU32, NonZeroUsize};

use critical_section::Mutex;
use defmt::*;
use defmt_rtt as _;
// use embedded_hal::adc::OneShot;
// use embedded_hal::digital::v2::OutputPin;
use embedded_hal::PwmPin;
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;

use bsp::entry;
use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio,
    pac::{self, interrupt},
    pwm,
    sio::Sio,
    watchdog::Watchdog,
};

const PREV_WEIGHT: u32 = 2;
const DEAD_ZONE: u32 = 1500;
const DEAD_ZONE_DIST: u32 = 4;

type PwmInputPin<GpioPin> = gpio::Pin<GpioPin, gpio::FunctionPwm, gpio::PullNone>;
type PwmSlice<Pwm> = pwm::Slice<Pwm, pwm::InputHighRunning>;

struct GlobalPins(
    PwmInputPin<gpio::bank0::Gpio3>,
    PwmSlice<pwm::Pwm1>,
    // b: PwmInputPin<gpio::bank0::Gpio4>,
    // c: PwmInputPin<gpio::bank0::Gpio5>,
);

static PINS: Mutex<RefCell<Option<GlobalPins>>> = Mutex::new(RefCell::new(None));
// type InputPinA = PwmInputPin<gpio::bank0::Gpio3>

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
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

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Init PWMs
    let pwm_slices = pwm::Slices::new(pac.PWM, &mut pac.RESETS);

    // Configure PWM0 slice
    // The PWM slice clock should only run when the input is high (InputHighRunning)
    let mut pwm_a: pwm::Slice<_, pwm::InputHighRunning> = pwm_slices.pwm1.into_mode();

    // Divide the 125 MHz system clock by 125 to give a 1 MHz PWM slice clock (1 us per tick)
    // pwm_a.set_div_int(125);
    pwm_a.enable();

    // let _ = pins.gpio2.into_function::<gpio::FunctionPwm>();
    // let _ = pins.gpio2.into_function::<gpio::FunctionPwm>();
    let pin_a = pins.gpio3.reconfigure();
    pin_a.set_interrupt_enabled(gpio::Interrupt::EdgeLow, true);
    // pin_a.set_interrupt_enabled(gpio::Interrupt::EdgeLow, true);

    let chan = &mut pwm_a.channel_b;
    chan.enable();

    // Give away our pins by moving them into the `GLOBAL_PINS` variable.
    // We won't need to access them in the main thread again
    critical_section::with(|cs| {
        PINS.borrow(cs).replace(Some(GlobalPins(pin_a, pwm_a)));
    });

    // Unmask the IO_BANK0 IRQ so that the NVIC interrupt controller
    // will jump to the interrupt function when the interrupt occurs.
    // We do this last so that the interrupt can't go off while
    // it is in the middle of being configured
    unsafe {
        pac::NVIC::unmask(pac::Interrupt::IO_IRQ_BANK0);
    }

    loop {
        cortex_m::asm::wfi();
        // let pulse_width = pwm_a.get_counter();
        // info!("pulse_width_us = {}", pulse_width);
        // pwm_a.set_counter(0);
        // info!("on!");
        // led_pin.set_high().unwrap();
        // info!("off!");
        // led_pin.set_low().unwrap();
        // delay.delay_ms(500);
    }
}

fn filter_pulse(next: u32, prev: Option<NonZeroU32>) -> Option<NonZeroU32> {
    // const X: u32 = 1000;
    if let Some(prev) = prev {
        let weighted = ((prev.get() * PREV_WEIGHT) + next) / (PREV_WEIGHT + 1);
        NonZeroU32::new(weighted)
        // let weighted = ((prev.get() * PREV_WEIGHT * X) + next * X) / (PREV_WEIGHT + 1);
        // if weighted - ((weighted / X) * X) >= (X / 10 * 5) {
        //     NonZeroU32::new(weighted / X + 1)
        // } else {
        //     NonZeroU32::new(weighted / X)
        // }
    } else {
        NonZeroU32::new(next)
    }
}

fn snap_to_dead_zone(pulse: NonZeroU32) -> NonZeroU32 {
    match DEAD_ZONE.abs_diff(pulse.get()) {
        x if x <= DEAD_ZONE_DIST => NonZeroU32::new(DEAD_ZONE).unwrap(),
        _ => pulse,
    }
}

#[interrupt]
fn IO_IRQ_BANK0() {
    // The `#[interrupt]` attribute covertly converts this to `&'static mut Option<LedAndInput>`
    static mut INPUT: Option<GlobalPins> = None;
    static mut PREV: Option<NonZeroU32> = None;

    if INPUT.is_none() {
        critical_section::with(|cs| {
            *INPUT = PINS.borrow(cs).take();
        });
    }

    let Some(GlobalPins(input_a, pwm_a)) = INPUT else {
        return;
    };

    // Check if the interrupt source is from the input pin going from high-to-low.
    // Note: this will always be true in this example, as that is the only enabled GPIO interrupt source
    if input_a.interrupt_status(gpio::Interrupt::EdgeLow) {
        // Read the width of the last pulse from the PWM Slice counter
        let pulse_width_us = pwm_a.get_counter() as u32;
        let pulse = filter_pulse(pulse_width_us, *PREV).map(snap_to_dead_zone);
        *PREV = pulse;

        info!(
            "pulse_width_us = {:04}, weighted = {:04}",
            pulse_width_us,
            pulse.map(|x| x.get()).unwrap_or(0)
        );

        // Reset
        pwm_a.set_counter(0);
        input_a.clear_interrupt(gpio::Interrupt::EdgeLow);
    }
}
