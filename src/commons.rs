pub const fn clock_top_phase(cpu_hz: u32, pwm_hz: u32, prescaler: u32) -> u8 {
    (cpu_hz / pwm_hz / prescaler / 2) as u8
}

pub const fn clock_top_fast(cpu_hz: u32, pwm_hz: u32, prescaler: u32) -> u8 {
    (cpu_hz / pwm_hz / prescaler - 1) as u8
}

pub trait Temp {
    fn temp(&self) -> Self;
}

pub fn interpolate_curve(approx_temp: u8, curves: &[[u8; 2]]) -> u8 {
    for n in 0..curves.len() {
        if curves[n][0] > approx_temp {
            // Linear interpolation between the duty cycles at specific temperatures
            // Calculation optimized such that no floats need to be used
            let [h_temp, h_duty] = curves[n];
            let [l_temp, l_duty] = curves[n - 1];

            let left: u16 = l_duty as u16 * (h_temp - approx_temp) as u16;
            let right: u16 = h_duty as u16 * (approx_temp - l_temp) as u16;
            let sum: u16 = left + right;
            let divisor: u16 = (h_temp - l_temp).into();

            // Should never fail, as long as the values in curves make sense
            return (sum / divisor).try_into().unwrap();
        }
    }
    100
}

pub trait Controller {
    fn update(&mut self);
}

pub trait PwmPhaseInit<TC> {
    fn setup(tc: TC, target_f: u32) -> Self;
}

pub trait SetDuty {
    fn set_duty(&self, percent: u8);
}

pub trait FanCurve<T> {
    fn curve(temperature: T) -> u8;
}
