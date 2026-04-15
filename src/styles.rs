use std::collections::BTreeMap;

pub const ANSI_BACKGROUND_OFFSET: u8 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StyleCode {
    pub open: u8,
    pub close: u8,
}

impl StyleCode {
    pub const fn new(open: u8, close: u8) -> Self {
        Self { open, close }
    }

    pub fn open_escape(self) -> String {
        format!("\x1B[{}m", self.open)
    }

    pub fn close_escape(self) -> String {
        format!("\x1B[{}m", self.close)
    }
}

macro_rules! style_code {
    ($name:ident, $open:literal, $close:literal) => {
        pub const $name: StyleCode = StyleCode::new($open, $close);
    };
}

style_code!(RESET, 0, 0);
style_code!(BOLD, 1, 22);
style_code!(DIM, 2, 22);
style_code!(ITALIC, 3, 23);
style_code!(UNDERLINE, 4, 24);
style_code!(OVERLINE, 53, 55);
style_code!(INVERSE, 7, 27);
style_code!(HIDDEN, 8, 28);
style_code!(STRIKETHROUGH, 9, 29);

style_code!(BLACK, 30, 39);
style_code!(RED, 31, 39);
style_code!(GREEN, 32, 39);
style_code!(YELLOW, 33, 39);
style_code!(BLUE, 34, 39);
style_code!(MAGENTA, 35, 39);
style_code!(CYAN, 36, 39);
style_code!(WHITE, 37, 39);
style_code!(BLACK_BRIGHT, 90, 39);
pub const GRAY: StyleCode = BLACK_BRIGHT;
pub const GREY: StyleCode = BLACK_BRIGHT;
style_code!(RED_BRIGHT, 91, 39);
style_code!(GREEN_BRIGHT, 92, 39);
style_code!(YELLOW_BRIGHT, 93, 39);
style_code!(BLUE_BRIGHT, 94, 39);
style_code!(MAGENTA_BRIGHT, 95, 39);
style_code!(CYAN_BRIGHT, 96, 39);
style_code!(WHITE_BRIGHT, 97, 39);

style_code!(BG_BLACK, 40, 49);
style_code!(BG_RED, 41, 49);
style_code!(BG_GREEN, 42, 49);
style_code!(BG_YELLOW, 43, 49);
style_code!(BG_BLUE, 44, 49);
style_code!(BG_MAGENTA, 45, 49);
style_code!(BG_CYAN, 46, 49);
style_code!(BG_WHITE, 47, 49);
style_code!(BG_BLACK_BRIGHT, 100, 49);
pub const BG_GRAY: StyleCode = BG_BLACK_BRIGHT;
pub const BG_GREY: StyleCode = BG_BLACK_BRIGHT;
style_code!(BG_RED_BRIGHT, 101, 49);
style_code!(BG_GREEN_BRIGHT, 102, 49);
style_code!(BG_YELLOW_BRIGHT, 103, 49);
style_code!(BG_BLUE_BRIGHT, 104, 49);
style_code!(BG_MAGENTA_BRIGHT, 105, 49);
style_code!(BG_CYAN_BRIGHT, 106, 49);
style_code!(BG_WHITE_BRIGHT, 107, 49);

pub const MODIFIER_NAMES: &[&str] = &[
    "reset",
    "bold",
    "dim",
    "italic",
    "underline",
    "overline",
    "inverse",
    "hidden",
    "strikethrough",
];

pub const FOREGROUND_COLOR_NAMES: &[&str] = &[
    "black",
    "red",
    "green",
    "yellow",
    "blue",
    "magenta",
    "cyan",
    "white",
    "blackBright",
    "gray",
    "grey",
    "redBright",
    "greenBright",
    "yellowBright",
    "blueBright",
    "magentaBright",
    "cyanBright",
    "whiteBright",
];

pub const BACKGROUND_COLOR_NAMES: &[&str] = &[
    "bgBlack",
    "bgRed",
    "bgGreen",
    "bgYellow",
    "bgBlue",
    "bgMagenta",
    "bgCyan",
    "bgWhite",
    "bgBlackBright",
    "bgGray",
    "bgGrey",
    "bgRedBright",
    "bgGreenBright",
    "bgYellowBright",
    "bgBlueBright",
    "bgMagentaBright",
    "bgCyanBright",
    "bgWhiteBright",
];

pub const COLOR_NAMES: &[&str] = &[
    "black",
    "red",
    "green",
    "yellow",
    "blue",
    "magenta",
    "cyan",
    "white",
    "blackBright",
    "gray",
    "grey",
    "redBright",
    "greenBright",
    "yellowBright",
    "blueBright",
    "magentaBright",
    "cyanBright",
    "whiteBright",
    "bgBlack",
    "bgRed",
    "bgGreen",
    "bgYellow",
    "bgBlue",
    "bgMagenta",
    "bgCyan",
    "bgWhite",
    "bgBlackBright",
    "bgGray",
    "bgGrey",
    "bgRedBright",
    "bgGreenBright",
    "bgYellowBright",
    "bgBlueBright",
    "bgMagentaBright",
    "bgCyanBright",
    "bgWhiteBright",
];

pub const CODE_PAIRS: &[(u8, u8)] = &[
    (RESET.open, RESET.close),
    (BOLD.open, BOLD.close),
    (DIM.open, DIM.close),
    (ITALIC.open, ITALIC.close),
    (UNDERLINE.open, UNDERLINE.close),
    (OVERLINE.open, OVERLINE.close),
    (INVERSE.open, INVERSE.close),
    (HIDDEN.open, HIDDEN.close),
    (STRIKETHROUGH.open, STRIKETHROUGH.close),
    (BLACK.open, BLACK.close),
    (RED.open, RED.close),
    (GREEN.open, GREEN.close),
    (YELLOW.open, YELLOW.close),
    (BLUE.open, BLUE.close),
    (MAGENTA.open, MAGENTA.close),
    (CYAN.open, CYAN.close),
    (WHITE.open, WHITE.close),
    (BLACK_BRIGHT.open, BLACK_BRIGHT.close),
    (GRAY.open, GRAY.close),
    (GREY.open, GREY.close),
    (RED_BRIGHT.open, RED_BRIGHT.close),
    (GREEN_BRIGHT.open, GREEN_BRIGHT.close),
    (YELLOW_BRIGHT.open, YELLOW_BRIGHT.close),
    (BLUE_BRIGHT.open, BLUE_BRIGHT.close),
    (MAGENTA_BRIGHT.open, MAGENTA_BRIGHT.close),
    (CYAN_BRIGHT.open, CYAN_BRIGHT.close),
    (WHITE_BRIGHT.open, WHITE_BRIGHT.close),
    (BG_BLACK.open, BG_BLACK.close),
    (BG_RED.open, BG_RED.close),
    (BG_GREEN.open, BG_GREEN.close),
    (BG_YELLOW.open, BG_YELLOW.close),
    (BG_BLUE.open, BG_BLUE.close),
    (BG_MAGENTA.open, BG_MAGENTA.close),
    (BG_CYAN.open, BG_CYAN.close),
    (BG_WHITE.open, BG_WHITE.close),
    (BG_BLACK_BRIGHT.open, BG_BLACK_BRIGHT.close),
    (BG_GRAY.open, BG_GRAY.close),
    (BG_GREY.open, BG_GREY.close),
    (BG_RED_BRIGHT.open, BG_RED_BRIGHT.close),
    (BG_GREEN_BRIGHT.open, BG_GREEN_BRIGHT.close),
    (BG_YELLOW_BRIGHT.open, BG_YELLOW_BRIGHT.close),
    (BG_BLUE_BRIGHT.open, BG_BLUE_BRIGHT.close),
    (BG_MAGENTA_BRIGHT.open, BG_MAGENTA_BRIGHT.close),
    (BG_CYAN_BRIGHT.open, BG_CYAN_BRIGHT.close),
    (BG_WHITE_BRIGHT.open, BG_WHITE_BRIGHT.close),
];

#[derive(Debug, Clone, Copy)]
pub struct ModifierStyles {
    pub reset: StyleCode,
    pub bold: StyleCode,
    pub dim: StyleCode,
    pub italic: StyleCode,
    pub underline: StyleCode,
    pub overline: StyleCode,
    pub inverse: StyleCode,
    pub hidden: StyleCode,
    pub strikethrough: StyleCode,
}

impl ModifierStyles {
    pub const fn new() -> Self {
        Self {
            reset: RESET,
            bold: BOLD,
            dim: DIM,
            italic: ITALIC,
            underline: UNDERLINE,
            overline: OVERLINE,
            inverse: INVERSE,
            hidden: HIDDEN,
            strikethrough: STRIKETHROUGH,
        }
    }

    pub const fn names(&self) -> &'static [&'static str] {
        MODIFIER_NAMES
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColorStyles {
    pub black: StyleCode,
    pub red: StyleCode,
    pub green: StyleCode,
    pub yellow: StyleCode,
    pub blue: StyleCode,
    pub magenta: StyleCode,
    pub cyan: StyleCode,
    pub white: StyleCode,
    pub black_bright: StyleCode,
    pub gray: StyleCode,
    pub grey: StyleCode,
    pub red_bright: StyleCode,
    pub green_bright: StyleCode,
    pub yellow_bright: StyleCode,
    pub blue_bright: StyleCode,
    pub magenta_bright: StyleCode,
    pub cyan_bright: StyleCode,
    pub white_bright: StyleCode,
    pub close: &'static str,
}

impl ColorStyles {
    pub const fn new() -> Self {
        Self {
            black: BLACK,
            red: RED,
            green: GREEN,
            yellow: YELLOW,
            blue: BLUE,
            magenta: MAGENTA,
            cyan: CYAN,
            white: WHITE,
            black_bright: BLACK_BRIGHT,
            gray: GRAY,
            grey: GREY,
            red_bright: RED_BRIGHT,
            green_bright: GREEN_BRIGHT,
            yellow_bright: YELLOW_BRIGHT,
            blue_bright: BLUE_BRIGHT,
            magenta_bright: MAGENTA_BRIGHT,
            cyan_bright: CYAN_BRIGHT,
            white_bright: WHITE_BRIGHT,
            close: "\x1B[39m",
        }
    }

    pub const fn names(&self) -> &'static [&'static str] {
        FOREGROUND_COLOR_NAMES
    }

    pub fn ansi(&self, code: u8) -> String {
        wrap_ansi16(0, code)
    }

    pub fn ansi256(&self, code: u8) -> String {
        wrap_ansi256(0, code)
    }

    pub fn ansi16m(&self, red: u8, green: u8, blue: u8) -> String {
        wrap_ansi16m(0, red, green, blue)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BgColorStyles {
    pub bg_black: StyleCode,
    pub bg_red: StyleCode,
    pub bg_green: StyleCode,
    pub bg_yellow: StyleCode,
    pub bg_blue: StyleCode,
    pub bg_magenta: StyleCode,
    pub bg_cyan: StyleCode,
    pub bg_white: StyleCode,
    pub bg_black_bright: StyleCode,
    pub bg_gray: StyleCode,
    pub bg_grey: StyleCode,
    pub bg_red_bright: StyleCode,
    pub bg_green_bright: StyleCode,
    pub bg_yellow_bright: StyleCode,
    pub bg_blue_bright: StyleCode,
    pub bg_magenta_bright: StyleCode,
    pub bg_cyan_bright: StyleCode,
    pub bg_white_bright: StyleCode,
    pub close: &'static str,
}

impl BgColorStyles {
    pub const fn new() -> Self {
        Self {
            bg_black: BG_BLACK,
            bg_red: BG_RED,
            bg_green: BG_GREEN,
            bg_yellow: BG_YELLOW,
            bg_blue: BG_BLUE,
            bg_magenta: BG_MAGENTA,
            bg_cyan: BG_CYAN,
            bg_white: BG_WHITE,
            bg_black_bright: BG_BLACK_BRIGHT,
            bg_gray: BG_GRAY,
            bg_grey: BG_GREY,
            bg_red_bright: BG_RED_BRIGHT,
            bg_green_bright: BG_GREEN_BRIGHT,
            bg_yellow_bright: BG_YELLOW_BRIGHT,
            bg_blue_bright: BG_BLUE_BRIGHT,
            bg_magenta_bright: BG_MAGENTA_BRIGHT,
            bg_cyan_bright: BG_CYAN_BRIGHT,
            bg_white_bright: BG_WHITE_BRIGHT,
            close: "\x1B[49m",
        }
    }

    pub const fn names(&self) -> &'static [&'static str] {
        BACKGROUND_COLOR_NAMES
    }

    pub fn ansi(&self, code: u8) -> String {
        wrap_ansi16(ANSI_BACKGROUND_OFFSET, code)
    }

    pub fn ansi256(&self, code: u8) -> String {
        wrap_ansi256(ANSI_BACKGROUND_OFFSET, code)
    }

    pub fn ansi16m(&self, red: u8, green: u8, blue: u8) -> String {
        wrap_ansi16m(ANSI_BACKGROUND_OFFSET, red, green, blue)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AnsiStyles {
    pub modifier: ModifierStyles,
    pub color: ColorStyles,
    pub bg_color: BgColorStyles,
}

impl AnsiStyles {
    pub const fn new() -> Self {
        Self {
            modifier: ModifierStyles::new(),
            color: ColorStyles::new(),
            bg_color: BgColorStyles::new(),
        }
    }

    pub fn codes(&self) -> BTreeMap<u8, u8> {
        codes()
    }

    pub const fn color_names(&self) -> &'static [&'static str] {
        COLOR_NAMES
    }

    pub fn rgb_to_ansi256(&self, red: u8, green: u8, blue: u8) -> u8 {
        rgb_to_ansi256(red, green, blue)
    }

    pub fn hex_to_rgb(&self, hex: impl AsRef<str>) -> [u8; 3] {
        hex_to_rgb(hex)
    }

    pub fn hex_to_ansi256(&self, hex: impl AsRef<str>) -> u8 {
        hex_to_ansi256(hex)
    }

    pub fn ansi256_to_ansi(&self, code: u8) -> u8 {
        ansi256_to_ansi(code)
    }

    pub fn rgb_to_ansi(&self, red: u8, green: u8, blue: u8) -> u8 {
        rgb_to_ansi(red, green, blue)
    }

    pub fn hex_to_ansi(&self, hex: impl AsRef<str>) -> u8 {
        hex_to_ansi(hex)
    }
}

pub const ANSI_STYLES: AnsiStyles = AnsiStyles::new();

pub fn wrap_ansi16(offset: u8, code: u8) -> String {
    format!("\x1B[{}m", code + offset)
}

pub fn wrap_ansi256(offset: u8, code: u8) -> String {
    format!("\x1B[{};5;{}m", 38 + offset, code)
}

pub fn wrap_ansi16m(offset: u8, red: u8, green: u8, blue: u8) -> String {
    format!("\x1B[{};2;{};{};{}m", 38 + offset, red, green, blue)
}

pub fn codes() -> BTreeMap<u8, u8> {
    CODE_PAIRS.iter().copied().collect()
}

pub fn rgb_to_ansi256(red: u8, green: u8, blue: u8) -> u8 {
    if red == green && green == blue {
        if red < 8 {
            return 16;
        }

        if red > 248 {
            return 231;
        }

        return ((((red - 8) as f64) / 247.0) * 24.0).round() as u8 + 232;
    }

    16 + (36 * ((red as f64 / 255.0) * 5.0).round() as u8)
        + (6 * ((green as f64 / 255.0) * 5.0).round() as u8)
        + ((blue as f64 / 255.0) * 5.0).round() as u8
}

pub fn hex_to_rgb(hex: impl AsRef<str>) -> [u8; 3] {
    let mut color = match first_hex_match(hex.as_ref()) {
        Some(color) => color,
        None => return [0, 0, 0],
    };

    if color.len() == 3 {
        color = color.chars().flat_map(|ch| [ch, ch]).collect();
    }

    let integer = match u32::from_str_radix(&color, 16) {
        Ok(value) => value,
        Err(_) => return [0, 0, 0],
    };

    [
        ((integer >> 16) & 0xFF) as u8,
        ((integer >> 8) & 0xFF) as u8,
        (integer & 0xFF) as u8,
    ]
}

pub fn hex_to_ansi256(hex: impl AsRef<str>) -> u8 {
    let [red, green, blue] = hex_to_rgb(hex);
    rgb_to_ansi256(red, green, blue)
}

pub fn ansi256_to_ansi(mut code: u8) -> u8 {
    if code < 8 {
        return 30 + code;
    }

    if code < 16 {
        return 90 + (code - 8);
    }

    let (red, green, blue) = if code >= 232 {
        let grayscale = (((code - 232) as f64 * 10.0) + 8.0) / 255.0;
        (grayscale, grayscale, grayscale)
    } else {
        code -= 16;

        let remainder = code % 36;
        let red = (code / 36) as f64 / 5.0;
        let green = (remainder / 6) as f64 / 5.0;
        let blue = (remainder % 6) as f64 / 5.0;

        (red, green, blue)
    };

    let value = red.max(green).max(blue) * 2.0;

    if value == 0.0 {
        return 30;
    }

    let mut result =
        30 + ((blue.round() as u8) << 2) + ((green.round() as u8) << 1) + red.round() as u8;

    if value == 2.0 {
        result += 60;
    }

    result
}

pub fn rgb_to_ansi(red: u8, green: u8, blue: u8) -> u8 {
    ansi256_to_ansi(rgb_to_ansi256(red, green, blue))
}

pub fn hex_to_ansi(hex: impl AsRef<str>) -> u8 {
    ansi256_to_ansi(hex_to_ansi256(hex))
}

fn first_hex_match(input: &str) -> Option<String> {
    let bytes = input.as_bytes();

    for len in [6, 3] {
        if bytes.len() < len {
            continue;
        }

        for window in bytes.windows(len) {
            if window.iter().all(|byte| byte.is_ascii_hexdigit()) {
                return Some(String::from_utf8_lossy(window).to_ascii_lowercase());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_codes_render_escape_sequences() {
        assert_eq!(RED.open_escape(), "\x1B[31m");
        assert_eq!(RED.close_escape(), "\x1B[39m");
        assert_eq!(BG_BLUE.open_escape(), "\x1B[44m");
        assert_eq!(BG_BLUE.close_escape(), "\x1B[49m");
    }

    #[test]
    fn group_helpers_match_js_behavior() {
        assert_eq!(ANSI_STYLES.color.ansi(31), "\x1B[31m");
        assert_eq!(ANSI_STYLES.color.ansi256(128), "\x1B[38;5;128m");
        assert_eq!(ANSI_STYLES.color.ansi16m(1, 2, 3), "\x1B[38;2;1;2;3m");
        assert_eq!(ANSI_STYLES.bg_color.ansi(31), "\x1B[41m");
        assert_eq!(ANSI_STYLES.bg_color.ansi256(128), "\x1B[48;5;128m");
        assert_eq!(ANSI_STYLES.bg_color.ansi16m(1, 2, 3), "\x1B[48;2;1;2;3m");
    }

    #[test]
    fn conversions_match_reference_logic() {
        assert_eq!(rgb_to_ansi256(0, 0, 0), 16);
        assert_eq!(rgb_to_ansi256(255, 255, 255), 231);
        assert_eq!(hex_to_rgb("#abc"), [170, 187, 204]);
        assert_eq!(hex_to_rgb("0xff8800"), [255, 136, 0]);
        assert_eq!(hex_to_rgb("invalid"), [0, 0, 0]);
        assert_eq!(hex_to_ansi256("#ff0000"), 196);
        assert_eq!(ansi256_to_ansi(196), 91);
        assert_eq!(rgb_to_ansi(255, 0, 0), 91);
        assert_eq!(hex_to_ansi("#ff0000"), 91);
    }

    #[test]
    fn codes_map_contains_open_close_pairs() {
        let codes = ANSI_STYLES.codes();
        assert_eq!(codes.get(&1), Some(&22));
        assert_eq!(codes.get(&31), Some(&39));
        assert_eq!(codes.get(&100), Some(&49));
    }
}
