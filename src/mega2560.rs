use arduino_hal::clock::MHz16;
use arduino_hal::hal::port::{PE0, PE1, PF0};

pub use arduino_hal::hal::prelude::*;
pub use arduino_hal::hal::Atmega as HalBoard;

use arduino_hal::port::mode::{Analog, Input, Output};
use arduino_hal::port::Pin;
use arduino_hal::Adc;

pub use avr_device::atmega2560 as mhal;
pub use avr_hal_generic::void;
pub use ufmt::{uWrite, uwrite};
pub use void::ResultVoidExt;

use arduino_hal::pac::USART0;
use arduino_hal::usart::Usart;

pub type Serial =
    Usart<USART0, avr_hal_generic::port::Pin<Input, PE0>, avr_hal_generic::port::Pin<Output, PE1>>;
pub type CoreClock = MHz16;
use arduino_hal::clock::Clock;

use mhal::tc0::tccr0b::CS0_A as FanPrescale;
use mhal::TC0 as FanPwmCounter;

use mhal::tc2::tccr2b::CS2_A as PumpPrescale;
use mhal::TC2 as PumpPwmCounter;

use num_traits::Float;

use fixed::types::I20F12;
type IFix = I20F12;

use crate::commons::{
    clock_top_phase, interpolate_curve, Controller, FanCurve, PwmPhaseInit, SetDuty, Temp,
};

pub struct PwmDevice<Tc> {
    tc: Tc,
    max_ticks: u8,
}

pub struct Board {
    #[cfg(feature = "fan")]
    fan: PwmDevice<FanPwmCounter>,
    #[cfg(feature = "pump")]
    pump: PwmDevice<PumpPwmCounter>,
    analog: Pin<Analog, PF0>,
    adc: Adc,
    serial: Serial,

    avg: IFix,
}

impl Board {
    pub fn new(target_f: u32) -> Board {
        let dp = arduino_hal::Peripherals::take().unwrap();
        let pins = arduino_hal::pins!(dp);
        let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

        #[cfg(feature = "fan")]
        let mut _t = pins.d4.into_output();
        #[cfg(feature = "pump")]
        let mut _t = pins.d9.into_output();

        let analog = pins.a0.into_analog_input(&mut adc);
        let serial = arduino_hal::default_serial!(dp, pins, 57600);

        #[cfg(feature = "fan")]
        let tc0 = dp.TC0;
        #[cfg(feature = "pump")]
        let tc2 = dp.TC2;
        Board {
            #[cfg(feature = "fan")]
            fan: PwmDevice::setup(tc0, target_f),
            #[cfg(feature = "pump")]
            pump: PwmDevice::setup(tc2, target_f),
            analog,
            adc,
            serial,
            avg: IFix::from_num(512),
        }
    }
}

impl Controller for Board {
    fn update(&mut self) {
        let measurement: IFix = self.analog.analog_read(&mut self.adc).into();
        self.avg = IFix::from_num(0.8) * self.avg + IFix::from_num(0.2) * measurement;
        let temp = self.avg.temp();

        let fan_calc = PwmDevice::<FanPwmCounter>::curve(temp.to_num());
        #[cfg(feature = "fan")]
        self.fan.set_duty(fan_calc);

        let pump_calc = PwmDevice::<PumpPwmCounter>::curve(temp.to_num());
        #[cfg(feature = "pump")]
        self.pump.set_duty(pump_calc);

        // debug info, will be optimized away if not compiled with serial
        let pre: i32 = temp.to_num();
        let aft: i32 = (temp * 100).to_num::<i32>() - pre * 100;

        log!(
            self.serial,
            "temperature: {}.{}, Fan: {}, Pump: {}",
            pre,
            aft,
            fan_calc,
            pump_calc
        );
    }
}

impl Temp for IFix {
    fn temp(&self) -> IFix {
        // Linear approx of voltage to temperature function.
        // More than enough for my application
        //
        // Based on this equation:
        // R = R_t + 1 + \alpha(T - T_t)
        // Where R_t is 10K at T_t = 25C
        // This corresponds to an analog read of 512 when using a 10k reference resistor
        //
        // This avoids using the complex Steinhart-Hart equation and `ln`
        //
        // -self + 1023 - 512 - 1
        (-self + IFix::from_num(510)) / -10 + IFix::from_num(25)
    }
}

impl SetDuty for PwmDevice<FanPwmCounter> {
    fn set_duty(&self, percent: u8) {
        let something = (self.max_ticks as u16 * percent as u16) / 100;
        self.tc.ocr0b.write(|w| w.bits(something as u8));
    }
}

impl FanCurve<u8> for PwmDevice<FanPwmCounter> {
    fn curve(approx_temp: u8) -> u8 {
        let steps = [[0, 0], [25, 10], [30, 20], [45, 100], [100, 100]];
        interpolate_curve(approx_temp, &steps)
    }
}

impl crate::commons::PwmPhaseInit<FanPwmCounter> for PwmDevice<FanPwmCounter> {
    fn setup(tc: FanPwmCounter, target_f: u32) -> PwmDevice<FanPwmCounter> {
        let prescale = best_prescaler!(FanPrescale, CoreClock::FREQ, target_f);

        let max_ticks: u8 = {
            let prescale_i: u16 = prescale_to_u16!(FanPrescale)(prescale);
            clock_top_phase(CoreClock::FREQ, target_f, prescale_i.into())
        };

        // 8bit: set Phase Correct with OCRA mode (5)
        tc.tccr0a.write(|w| {
            w.com0b().match_clear();
            w.wgm0().pwm_phase() // Select Phase correct PWM
        });

        tc.tccr0b.write(|w| {
            w.wgm02().bit(true); // select Phase correct PWM with OCR0A set top
            w.cs0().variant(prescale) // Set prescaler
        });

        // set TOP comparison register
        tc.ocr0a.write(|w| w.bits(max_ticks));

        PwmDevice { tc, max_ticks }
    }
}

impl crate::commons::PwmPhaseInit<PumpPwmCounter> for PwmDevice<PumpPwmCounter> {
    fn setup(tc: PumpPwmCounter, target_f: u32) -> PwmDevice<PumpPwmCounter> {
        let prescale = best_prescaler!(PumpPrescale, CoreClock::FREQ, target_f);

        let max_ticks: u8 = {
            let prescale_i: u16 = prescale_to_u16!(PumpPrescale)(prescale);
            clock_top_phase(CoreClock::FREQ, target_f, prescale_i.into())
        };

        // 8bit: set Phase Correct with OCRA mode (5)
        tc.tccr2a.write(|w| {
            w.com2b().match_clear();
            w.wgm2().pwm_phase() // Select Phase correct PWM
        });

        tc.tccr2b.write(|w| {
            w.wgm22().bit(true); // select Phase correct PWM with OCR0A set top
            w.cs2().variant(prescale) // Set prescaler
        });

        // set TOP comparison register
        tc.ocr2a.write(|w| w.bits(max_ticks));

        PwmDevice { tc, max_ticks }
    }
}

impl SetDuty for PwmDevice<PumpPwmCounter> {
    fn set_duty(&self, percent: u8) {
        let something = (self.max_ticks as u16 * percent as u16) / 100;
        self.tc.ocr2b.write(|w| w.bits(something as u8));
    }
}

impl FanCurve<u8> for PwmDevice<PumpPwmCounter> {
    fn curve(approx_temp: u8) -> u8 {
        let steps = [
            [0, 0],
            [25, 10],
            [30, 15],
            [35, 20],
            [40, 30],
            [50, 50],
            [55, 100],
            [100, 100],
        ];
        interpolate_curve(approx_temp, &steps)
    }
}

pub fn shh(input: u32) -> f32 {
    // Could be used for a more accurate voltage to temperature conversion
    // But it'll use more space, because it uses `ln` and floats
    let beta: f32 = 3950.0;
    let room_temp: f32 = 25.0;
    let resistor_at_room_temp: f32 = 10000.0;
    let temp: f32 = 1023.0 / input as f32 - 1.0;
    let temp = 10000.0 / temp;
    let temp: f32 = ((temp / resistor_at_room_temp) as f32).ln() / beta;
    let temp = 1.0 / (temp + (1.0 / (room_temp + 273.15)));
    temp - 273.15
}
