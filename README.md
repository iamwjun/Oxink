# Oxink

Oxink is a small Rust library for CLI rendering primitives. It exposes ANSI
style codes, foreground/background color helpers, and color conversion utilities
that are useful when building terminal output.

## Features

- ANSI modifier styles: bold, dim, italic, underline, overline, inverse, hidden,
  strikethrough, and reset.
- ANSI foreground and background colors, including bright color variants.
- Helpers for ANSI 16-color, 256-color, and truecolor escape sequences.
- RGB and Hex conversion helpers for ANSI 256-color and ANSI 16-color output.
- Zero runtime dependencies.

## Installation

Add Oxink to your `Cargo.toml`:

```toml
[dependencies]
oxink = "0.1.0"
```

If you are using the local repository directly:

```toml
[dependencies]
oxink = { path = "path/to/Oxink" }
```

## Usage

```rust
use oxink::styles::{ANSI_STYLES, BOLD, RED};

fn main() {
    let message = format!(
        "{}{}Error:{} something went wrong",
        BOLD.open_escape(),
        RED.open_escape(),
        RED.close_escape(),
    );

    println!("{}{}", message, BOLD.close_escape());

    let orange = ANSI_STYLES.color.ansi256(ANSI_STYLES.hex_to_ansi256("#ff8800"));
    println!("{orange}warning\x1B[39m");
}
```

Generate escape sequences directly:

```rust
use oxink::styles::ANSI_STYLES;

let fg = ANSI_STYLES.color.ansi16m(255, 136, 0);
let bg = ANSI_STYLES.bg_color.ansi256(236);

assert_eq!(fg, "\x1B[38;2;255;136;0m");
assert_eq!(bg, "\x1B[48;5;236m");
```

Convert colors:

```rust
use oxink::styles::{hex_to_ansi, hex_to_ansi256, hex_to_rgb, rgb_to_ansi};

assert_eq!(hex_to_rgb("#abc"), [170, 187, 204]);
assert_eq!(hex_to_ansi256("#ff0000"), 196);
assert_eq!(hex_to_ansi("#ff0000"), 91);
assert_eq!(rgb_to_ansi(255, 0, 0), 91);
```

## API Overview

### Style Codes

`StyleCode` stores an ANSI opening code and matching closing code:

```rust
use oxink::styles::UNDERLINE;

assert_eq!(UNDERLINE.open, 4);
assert_eq!(UNDERLINE.close, 24);
assert_eq!(UNDERLINE.open_escape(), "\x1B[4m");
assert_eq!(UNDERLINE.close_escape(), "\x1B[24m");
```

Available modifier constants:

```text
RESET, BOLD, DIM, ITALIC, UNDERLINE, OVERLINE, INVERSE, HIDDEN,
STRIKETHROUGH
```

Available foreground color constants:

```text
BLACK, RED, GREEN, YELLOW, BLUE, MAGENTA, CYAN, WHITE,
BLACK_BRIGHT, GRAY, GREY, RED_BRIGHT, GREEN_BRIGHT, YELLOW_BRIGHT,
BLUE_BRIGHT, MAGENTA_BRIGHT, CYAN_BRIGHT, WHITE_BRIGHT
```

Available background color constants:

```text
BG_BLACK, BG_RED, BG_GREEN, BG_YELLOW, BG_BLUE, BG_MAGENTA, BG_CYAN,
BG_WHITE, BG_BLACK_BRIGHT, BG_GRAY, BG_GREY, BG_RED_BRIGHT,
BG_GREEN_BRIGHT, BG_YELLOW_BRIGHT, BG_BLUE_BRIGHT, BG_MAGENTA_BRIGHT,
BG_CYAN_BRIGHT, BG_WHITE_BRIGHT
```

### Grouped Styles

`ANSI_STYLES` provides grouped access to modifiers, colors, background colors,
name lists, code maps, and conversion helpers:

```rust
use oxink::styles::ANSI_STYLES;

let names = ANSI_STYLES.color_names();
let codes = ANSI_STYLES.codes();

assert!(names.contains(&"red"));
assert_eq!(codes.get(&31), Some(&39));
```

### Escape Sequence Helpers

```rust
use oxink::styles::{wrap_ansi16, wrap_ansi16m, wrap_ansi256};

assert_eq!(wrap_ansi16(0, 31), "\x1B[31m");
assert_eq!(wrap_ansi256(0, 128), "\x1B[38;5;128m");
assert_eq!(wrap_ansi16m(0, 1, 2, 3), "\x1B[38;2;1;2;3m");
```

For background colors, pass the background offset:

```rust
use oxink::styles::{wrap_ansi256, ANSI_BACKGROUND_OFFSET};

assert_eq!(wrap_ansi256(ANSI_BACKGROUND_OFFSET, 128), "\x1B[48;5;128m");
```

## Development

Run the test suite:

```sh
cargo test
```

Format the code:

```sh
cargo fmt
```

Run Clippy:

```sh
cargo clippy --all-targets --all-features
```

## License

MIT
