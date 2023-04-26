`fan-control-rs`
==================
Personal project for building a automatic pwm fan and pump control using a 10k thermistor in a liquid watercooling loop.
Intended to be used with either two Attiny85 or at least one Arduino Mega 2560

## Usage
Build and flash the microcontrollers. Use a voltage bridge with a reference resistance of 10k for the Probe.
Attach the Probe to Analog Pin 0 (Atmega) or PB2 of the Attiny. 
Attach the Fan or Pump to pin PB1 of the Attinys, or Pins PE0 and PE1 for the Atmega. 

Flashing the Attiny85 with `ravedude` and the `usbasp` won't work out of the box. 
You could patch `ravedude` and install it from source: 

```bash
git clone https://github.com/Rahix/avr-hal.git
cp tiny.patch avr-hal/ravedude
cd avr-hal/ravedude
git apply tiny.patch

cargo install --path .
```

Alternatively, run 
```bash
avrdude -p t85 -cusbasp -P usb -U .\target\avr-attiny85\release\fan-controll-rs.elf
```

### Compile

#### Arduino Mega2560
```shell
cargo build --release --target avr-specs/avr-atmega2560.json --features pump,fan,phase_pwm,serial
```
#### Attiny85
```shell
cargo build --release --target avr-specs/avr-attiny85.json --features fan,phase_pwm
cargo build --release --target avr-specs/avr-attiny85.json --features pump,phase_pwm
```

Use `pump` and `fan` to select the fan curve. 
The `phase_pwm` and `simple_pwm` features can be used to select the type of pwm implementation.
Specifics are board dependent. Simple will allow you to use a more granular control of the duty cycle, but
my application doesn't need this amount of granularity.

You can enable serial output by using `serial` on the Atmega. The statement won't have an effect on the Tiny.

## License
Licensed under either of

 - Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 - MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
