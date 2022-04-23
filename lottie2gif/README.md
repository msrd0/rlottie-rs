# lottie2gif [![lottie2gif on crates.io](https://img.shields.io/crates/v/lottie2gif.svg)](https://crates.io/crates/lottie2gif) [![lottie2gif docs](https://img.shields.io/badge/docs-release-blue)](https://docs.msrd0.de/#lottie2gif) [![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://mit-license.org/)

This crate can be used as a binary as well as a library. To compile it as a binary, pass `--all-features` or `--feature clap` to cargo.

```
lottie2gif 

Convert lottie files GIF

USAGE:
    lottie2gif [OPTIONS] <lottieFileName> [bgColor]

ARGS:
    <lottieFileName>    The location of the lottie file
    <bgColor>           The background color in hexadecimal format [default: 0]

OPTIONS:
    -h, --help               Print help information
        --non-transparent    Disable background transparency
    -o, --out <output>       The output file
```
