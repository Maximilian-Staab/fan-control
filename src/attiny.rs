pub use arduino_hal::hal::Attiny as HalBoard;
pub use avr_device::attiny85 as mhal;

pub type CoreClock = arduino_hal::clock::MHz8;
use arduino_hal::clock::Clock;

use arduino_hal::hal::port::PB2;

pub use arduino_hal::hal::prelude::*;

use arduino_hal::{
    hal::Adc,
    port::{mode::Analog, Pin},
};

use mhal::{tc0::tccr0b::CS0_A as Prescale, TC0 as PwmCounter};

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
    pwm: PwmDevice<PwmCounter>,
    analog: Pin<Analog, PB2>,
    adc: Adc<CoreClock>,

    avg: IFix,
}

impl Board {
    pub fn new(target_f: u32) -> Board {
        let dp = arduino_hal::Peripherals::take().unwrap();
        let pins = arduino_hal::pins!(dp);
        let mut adc = Adc::new(dp.ADC, Default::default());

        let mut _t = pins.d1.into_output();

        let analog = pins.d2.into_analog_input(&mut adc);

        let tc = dp.TC0;
        Board {
            pwm: PwmDevice::setup(tc, target_f),
            analog,
            adc,
            avg: IFix::from_num(512),
        }
    }
}

impl Controller for Board {
    fn update(&mut self) {
        let measurement: IFix = self.analog.analog_read(&mut self.adc).into();
        self.avg = IFix::from_num(0.8) * self.avg + IFix::from_num(0.2) * measurement;
        let temp = self.avg.temp();
        let calc = PwmDevice::<PwmCounter>::curve(temp.to_num());
        self.pwm.set_duty(calc);
    }
}

impl Temp for IFix {
    fn temp(&self) -> IFix {
        (-self + IFix::from_num(510)) / -10 + IFix::from_num(25)
    }
}

impl SetDuty for PwmDevice<PwmCounter> {
    fn set_duty(&self, percent: u8) {
        let something = (self.max_ticks as u16 * percent as u16) / 100;
        self.tc.ocr0b.write(|w| w.bits(something as u8));
    }
}

impl crate::commons::PwmPhaseInit<PwmCounter> for PwmDevice<PwmCounter> {
    fn setup(tc: PwmCounter, target_f: u32) -> PwmDevice<PwmCounter> {
        let prescale = best_prescaler!(Prescale, CoreClock::FREQ, target_f);
        let max_ticks: u8 = {
            let prescale_i: u16 = prescale_to_u16!(Prescale)(prescale);
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

impl FanCurve<u8> for PwmDevice<PwmCounter> {
    fn curve(approx_temp: u8) -> u8 {
        #[cfg(feature = "fan")]
        let steps = [[0, 0], [25, 10], [30, 20], [45, 100], [100, 100]];
        #[cfg(feature = "pump")]
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
