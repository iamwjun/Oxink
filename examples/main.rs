#[cfg(unix)]
use std::io;

#[cfg(unix)]
fn main() -> io::Result<()> {
    unix_demo::run()
}

#[cfg(not(unix))]
fn main() {
    eprintln!("This example currently supports Unix-like terminals only.");
}

#[cfg(unix)]
mod unix_demo {
    use oxink::input::{InputAction, InputTheme, KeyCode, KeyEvent, SlashInput, TerminalColor};
    use std::io::{self, Read, Write};
    use std::process::{Command, Stdio};

    const COMMANDS: &[(&str, &str)] = &[
        ("help", "Show available commands"),
        ("hello", "Insert a greeting"),
        ("history", "View recent commands"),
        ("theme", "Change terminal colors"),
        ("clear", "Clear the current session"),
        ("status", "Inspect the current environment"),
        ("search", "Find a command or snippet"),
        ("settings", "Open terminal preferences"),
        ("profile", "Switch the active profile"),
        ("export", "Export the current output"),
        ("reload", "Reload the current context"),
        ("version", "Show the current version"),
        ("quit", "Exit the current prompt"),
    ];

    const HELP_LINES: &[&str] = &[
        "Oxink input example",
        "",
        "Type text normally. Enter / as the first character to open suggestions.",
        "Up/Down selects, Enter/Tab applies the suggestion, Ctrl-V pastes.",
        "Ctrl-C or Ctrl-D exits the example.",
        "The demo clipboard starts with /help.",
        "",
    ];

    pub fn run() -> io::Result<()> {
        let _terminal_mode = TerminalMode::enter()?;
        let mut input = SlashInput::new(COMMANDS.iter().copied()).with_theme(
            InputTheme::ocean()
                .with_background_color(TerminalColor::Ansi256(236))
                .with_suggestion_background_color(TerminalColor::Ansi256(238))
                .with_selected_background_color(TerminalColor::Ansi256(31)),
        );
        let mut clipboard = String::from("/help");
        let mut status = String::from("Ready. Press Ctrl-V to paste the demo clipboard.");
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut stdin = stdin.lock();
        let mut stdout = stdout.lock();

        render(&mut stdout, &input, &clipboard, &status)?;

        loop {
            let Some(app_event) = read_event(&mut stdin)? else {
                continue;
            };

            match app_event {
                AppEvent::Quit => break,
                AppEvent::Key(event) => match input.handle_key(event) {
                    InputAction::None => {
                        status = format_status(&input);
                    }
                    InputAction::CopyRequested(value) => {
                        clipboard = value;
                        status = match clipboard.is_empty() {
                            true => "Copied an empty value.".to_string(),
                            false => format!("Copied: {}", preview(&clipboard)),
                        };
                    }
                    InputAction::PasteRequested => {
                        if clipboard.is_empty() {
                            status = "Clipboard is empty.".to_string();
                        } else {
                            input.handle_paste(&clipboard);
                            status = format!("Pasted: {}", preview(&clipboard));
                        }
                    }
                    InputAction::SuggestionApplied(value) => {
                        status = format!("Applied suggestion: {}", preview(&value));
                    }
                    InputAction::Submitted(value) => {
                        status = format!("Submitted: {}", preview(&value));
                    }
                },
            }

            render(&mut stdout, &input, &clipboard, &status)?;
        }

        Ok(())
    }

    enum AppEvent {
        Quit,
        Key(KeyEvent),
    }

    struct TerminalMode {
        original_state: String,
    }

    impl TerminalMode {
        fn enter() -> io::Result<Self> {
            let original_state = run_stty_capture(["-g"])?;
            run_stty(["raw", "-echo"])?;

            let mut stdout = io::stdout().lock();
            write!(stdout, "\x1B[?25l")?;
            stdout.flush()?;

            Ok(Self {
                original_state: original_state.trim().to_string(),
            })
        }
    }

    impl Drop for TerminalMode {
        fn drop(&mut self) {
            let _ = run_stty([self.original_state.as_str()]);

            let mut stdout = io::stdout().lock();
            let _ = write!(stdout, "\x1B[2J\x1B[H\x1B[?25h");
            let _ = stdout.flush();
        }
    }

    fn render<W: Write>(
        output: &mut W,
        input: &SlashInput,
        clipboard: &str,
        status: &str,
    ) -> io::Result<()> {
        let view = input.render_with_terminal_width(terminal_columns()?);

        write!(output, "\x1B[2J\x1B[H")?;
        for line in HELP_LINES {
            write_line(output, line)?;
        }
        for line in &view.lines {
            write_line(output, line)?;
        }
        write_line(output, "")?;
        write_line(output, &format!("Value: {}", preview(input.value())))?;
        write_line(output, &format!("Clipboard: {}", preview(clipboard)))?;
        write_line(output, &format!("Status: {status}"))?;

        let row = HELP_LINES.len() + view.cursor_row + 1;
        let column = view.cursor_column + 1;
        write!(output, "\x1B[{row};{column}H\x1B[?25h")?;
        output.flush()
    }

    fn read_event<R: Read>(reader: &mut R) -> io::Result<Option<AppEvent>> {
        let byte = match read_byte(reader)? {
            Some(byte) => byte,
            None => return Ok(None),
        };

        let event = match byte {
            0x04 => AppEvent::Quit,
            0x03 => AppEvent::Quit,
            0x16 => AppEvent::Key(KeyEvent::ctrl(KeyCode::Char('v'))),
            b'\r' | b'\n' => AppEvent::Key(KeyEvent::plain(KeyCode::Enter)),
            b'\t' => AppEvent::Key(KeyEvent::plain(KeyCode::Tab)),
            0x7f | 0x08 => AppEvent::Key(KeyEvent::plain(KeyCode::Backspace)),
            0x1b => match read_escape_sequence(reader)? {
                Some(event) => AppEvent::Key(event),
                None => return Ok(None),
            },
            byte if byte.is_ascii_control() => return Ok(None),
            byte => {
                let ch = read_char(reader, byte)?;
                AppEvent::Key(KeyEvent::plain(KeyCode::Char(ch)))
            }
        };

        Ok(Some(event))
    }

    fn read_escape_sequence<R: Read>(reader: &mut R) -> io::Result<Option<KeyEvent>> {
        let Some(prefix) = read_byte(reader)? else {
            return Ok(Some(KeyEvent::plain(KeyCode::Esc)));
        };

        let key = match prefix {
            b'[' => read_csi_sequence(reader)?,
            b'O' => read_ss3_sequence(reader)?,
            _ => KeyEvent::plain(KeyCode::Esc),
        };

        Ok(Some(key))
    }

    fn read_csi_sequence<R: Read>(reader: &mut R) -> io::Result<KeyEvent> {
        let Some(byte) = read_byte(reader)? else {
            return Ok(KeyEvent::plain(KeyCode::Esc));
        };

        let key = match byte {
            b'A' => KeyEvent::plain(KeyCode::Up),
            b'B' => KeyEvent::plain(KeyCode::Down),
            b'C' => KeyEvent::plain(KeyCode::Right),
            b'D' => KeyEvent::plain(KeyCode::Left),
            b'H' => KeyEvent::plain(KeyCode::Home),
            b'F' => KeyEvent::plain(KeyCode::End),
            b'1' | b'3' | b'4' | b'7' | b'8' => {
                let Some(suffix) = read_byte(reader)? else {
                    return Ok(KeyEvent::plain(KeyCode::Esc));
                };

                match (byte, suffix) {
                    (b'1', b'~') | (b'7', b'~') => KeyEvent::plain(KeyCode::Home),
                    (b'3', b'~') => KeyEvent::plain(KeyCode::Delete),
                    (b'4', b'~') | (b'8', b'~') => KeyEvent::plain(KeyCode::End),
                    _ => KeyEvent::plain(KeyCode::Esc),
                }
            }
            _ => KeyEvent::plain(KeyCode::Esc),
        };

        Ok(key)
    }

    fn read_ss3_sequence<R: Read>(reader: &mut R) -> io::Result<KeyEvent> {
        let Some(byte) = read_byte(reader)? else {
            return Ok(KeyEvent::plain(KeyCode::Esc));
        };

        let key = match byte {
            b'H' => KeyEvent::plain(KeyCode::Home),
            b'F' => KeyEvent::plain(KeyCode::End),
            _ => KeyEvent::plain(KeyCode::Esc),
        };

        Ok(key)
    }

    fn read_char<R: Read>(reader: &mut R, first_byte: u8) -> io::Result<char> {
        if first_byte.is_ascii() {
            return Ok(first_byte as char);
        }

        let width = utf8_width(first_byte)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid utf-8 lead byte"))?;
        let mut buffer = vec![0; width];
        buffer[0] = first_byte;

        if width > 1 {
            reader.read_exact(&mut buffer[1..])?;
        }

        let text = std::str::from_utf8(&buffer)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        text.chars()
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "empty utf-8 sequence"))
    }

    fn utf8_width(first_byte: u8) -> Option<usize> {
        match first_byte {
            0x00..=0x7f => Some(1),
            0xc0..=0xdf => Some(2),
            0xe0..=0xef => Some(3),
            0xf0..=0xf7 => Some(4),
            _ => None,
        }
    }

    fn read_byte<R: Read>(reader: &mut R) -> io::Result<Option<u8>> {
        let mut buffer = [0; 1];
        match reader.read(&mut buffer)? {
            0 => Ok(None),
            _ => Ok(Some(buffer[0])),
        }
    }

    fn write_line<W: Write>(output: &mut W, line: &str) -> io::Result<()> {
        write!(output, "{line}\r\n")
    }

    fn terminal_columns() -> io::Result<usize> {
        let size = run_stty_capture(["size"])?;
        let columns = size
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| io::Error::other("failed to parse terminal columns"))?;
        columns
            .parse::<usize>()
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }

    fn run_stty<I, S>(args: I) -> io::Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut command = Command::new("stty");
        command.stdin(Stdio::inherit());

        for arg in args {
            command.arg(arg.as_ref());
        }

        let status = command.status()?;
        if status.success() {
            Ok(())
        } else {
            Err(io::Error::other("stty command failed"))
        }
    }

    fn run_stty_capture<I, S>(args: I) -> io::Result<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut command = Command::new("stty");
        command.stdin(Stdio::inherit());

        for arg in args {
            command.arg(arg.as_ref());
        }

        let output = command.output()?;
        if !output.status.success() {
            return Err(io::Error::other("stty command failed"));
        }

        String::from_utf8(output.stdout)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }

    fn format_status(input: &SlashInput) -> String {
        match input.selected_command() {
            Some(command) if input.is_dropdown_visible() => {
                format!("Suggestion selected: /{command}")
            }
            _ if input.is_dropdown_visible() => "Suggestion list is open.".to_string(),
            _ => "Typing.".to_string(),
        }
    }

    fn preview(text: &str) -> String {
        if text.is_empty() {
            return "<empty>".to_string();
        }

        let preview: String = text
            .chars()
            .map(|ch| match ch {
                '\n' => ' ',
                '\r' => ' ',
                _ => ch,
            })
            .take(40)
            .collect();

        if text.chars().count() > 40 {
            format!("{preview}...")
        } else {
            preview
        }
    }
}
