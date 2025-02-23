# rlottie-sys [![rlottie-sys on crates.io](https://img.shields.io/crates/v/rlottie-sys.svg)](https://crates.io/crates/rlottie-sys) [![rlottie-sys docs](https://img.shields.io/badge/docs-release-blue)](https://docs.msrd0.de/#rlottie-sys) [![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://mit-license.org/)

Rust bindings to rlottie.

## Features

 - `vendor-samsung` (enabled by default): If rlottie cannot be found on the system download the samsung version of rlottie and compile it. You can force the use of the vendored code by setting `RLOTTIE_NO_PKG_CONFIG`.

 - `vendor-telegram`: If rlottie cannot be found on the system download the telegram version of rlottie and compile it. You can force the use of the vendored code by setting `RLOTTIE_NO_PKG_CONFIG`.

If both `vendor-samsung` and `vendor-telegram` are enabled, samsung has priority.
