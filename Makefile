mega:
	cargo build --release --target=avr-specs/avr-atmega2560.json

megarun:
	cargo run --release --target=avr-specs/avr-atmega2560.json

tiny:
	cargo build --release --target=avr-specs/avr-attiny85.json

tinyrun:
	cargo run --release --target=avr-specs/avr-attiny85.json
