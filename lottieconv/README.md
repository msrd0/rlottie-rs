# lottieconv [![lottieconv on crates.io](https://img.shields.io/crates/v/lottieconv.svg)](https://crates.io/crates/lottieconv) [![lottieconv docs](https://img.shields.io/badge/docs-release-blue)](https://docs.msrd0.de/#lottieconv) [![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://mit-license.org/)

This crate can act both as a library to convert lottie files to either webp or gif
files, as well as binaries to do the conversion.

## lottie2gif

This requires the features `clap` and `gif` to be active.

```
Convert lottie files GIF

Usage: lottie2gif [OPTIONS] <lottieFileName> [bgColor]

Arguments:
  <lottieFileName>  The location of the lottie file
  [bgColor]         The background color in hexadecimal format [default: 0]

Options:
      --non-transparent  Disable background transparency
  -o, --out <output>     The output file
  -h, --help             Print help information

```

## lottie2webp

This requires the features `clap` and `webp` to be active.

```
Convert lottie files WEBP

Usage: lottie2webp [OPTIONS] <lottieFileName>

Arguments:
  <lottieFileName>  The location of the lottie file

Options:
  -o, --out <output>  The output file
  -h, --help          Print help information

```
