#[macro_export]
macro_rules! best_prescaler {
    ($t:ty, $cf:expr, $tf:expr) => {{
        let cpu_f: u32 = $cf;
        let target_f: u32 = $tf;

        #[cfg(feature = "phase_pwm")]
        let divisor = 510;
        #[cfg(feature = "simple_pwm")]
        let divisor = 256;

        let approx_range = cpu_f / target_f / divisor;
        match approx_range {
            x if x == 0 => <$t>::DIRECT,
            x if x < 8 => <$t>::PRESCALE_8,
            x if x < 64 => <$t>::PRESCALE_64,
            x if x < 256 => <$t>::PRESCALE_256,
            x if x < 1024 => <$t>::PRESCALE_1024,
            _ => <$t>::DIRECT,
        }
    }};
}

#[macro_export]
macro_rules! best_prescaler_async {
    ($t:ty, $cf:expr, $tf:expr) => {
        let cpu_f: u32 = $cf;
        let target_f: u32 = $tf;

        #[cfg(feature = "phase_pwm")]
        let divisor = 510;
        #[cfg(feature = "simple_pwm")]
        let divisor = 256;

        let approx_range = cpu_f / target_f / divisor;
        match approx_range {
            x if x == 0 => <$t>::DIRECT,
            x if x < 8 => <$t>::PRESCALE_8,
            x if x < 32 => <$t>::PRESCALE_32,
            x if x < 64 => <$t>::PRESCALE_64,
            x if x < 128 => <$t>::PRESCALE_128,
            x if x < 256 => <$t>::PRESCALE_256,
            x if x < 1024 => <$t>::PRESCALE_1024,
            _ => <$t>::DIRECT,
        }
    };
}

#[macro_export]
macro_rules! prescale_to_u16 {
    ($t:ty) => {
        |x| match x {
            <$t>::DIRECT => 1,
            <$t>::PRESCALE_8 => 8,
            <$t>::PRESCALE_64 => 64,
            <$t>::PRESCALE_256 => 256,
            <$t>::PRESCALE_1024 => 1024,
            _ => 1,
        }
    };
}

#[macro_export]
macro_rules! prescale_async_to_u16 {
    ($t:ty) => {
        |x| match x {
            <$t>::DIRECT => 1,
            <$t>::PRESCALE_8 => 8,
            <$t>::PRESCALE_32 => 32,
            <$t>::PRESCALE_64 => 64,
            <$t>::PRESCALE_128 => 128,
            <$t>::PRESCALE_256 => 256,
            <$t>::PRESCALE_1024 => 1024,
            _ => 1,
        }
    };
}

#[macro_export]
macro_rules! log {
    ($serial:expr, $msg:expr $(, $tt:tt)*) => {
        // Avoids using the cfg flag every time you want to log something
        #[cfg(feature = "serial")]
        ufmt::uwriteln!(&mut $serial, $msg, $($tt),*).void_unwrap();
    };
}
