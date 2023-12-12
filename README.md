i2c-slave
=========

Rust project for the _SparkFun ProMini 5v_ that implements i2c slave. Master complementary repo is [here](https://github.com/kirillfx/avr-i2c-master-test).

I'm flashing it via USB programmer and serial interface for it works for me with 4800 baud in
serial monitor and 9600 on arduino board.

`RAVEDUDE_PORT` is configure with direnv `.envrc` file 

## Build Instructions

- Specify `RAVEDUDE_PORT` in `.envrc` if `direnv` is used. If you on linux with nix, change env var in `flake.nix`.

- Install prerequisites as described in the [`avr-hal` README] (`avr-gcc`, `avr-libc`, `avrdude`, [`ravedude`]).

- Run `cargo build` to build the firmware.

- Run `cargo run` to flash the firmware to a connected board.  If `ravedude`
   fails to detect your board, check its documentation at
   <https://crates.io/crates/ravedude>.

- `ravedude` will open a console session after flashing where you can interact
   with the UART console of your board.

[`avr-hal` README]: https://github.com/Rahix/avr-hal#readme
[`ravedude`]: https://crates.io/crates/ravedude

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
