use crate::styles::{ANSI_STYLES, BLUE, GRAY, WHITE};
use std::fmt;
use std::io::{self, Write};
use std::ops::{BitOr, BitOrAssign};

pub const DEFAULT_INPUT_WIDTH: usize = 28;
const DROPDOWN_TOP_PADDING: usize = 0;
const DROPDOWN_VERTICAL_SPACING: usize = 0;
const MAX_VISIBLE_DROPDOWN_OPTIONS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyModifiers(u8);

impl KeyModifiers {
    pub const NONE: Self = Self(0);
    pub const SHIFT: Self = Self(1 << 0);
    pub const CONTROL: Self = Self(1 << 1);
    pub const ALT: Self = Self(1 << 2);
    pub const SUPER: Self = Self(1 << 3);

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }
}

impl Default for KeyModifiers {
    fn default() -> Self {
        Self::NONE
    }
}

impl BitOr for KeyModifiers {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for KeyModifiers {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Backspace,
    Delete,
    Enter,
    Esc,
    Home,
    End,
    Left,
    Right,
    Up,
    Down,
    Tab,
    Char(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyEvent {
    pub const fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    pub const fn plain(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::NONE)
    }

    pub const fn ctrl(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::CONTROL)
    }

    pub const fn super_key(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::SUPER)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputAction {
    None,
    CopyRequested(String),
    PasteRequested,
    SuggestionApplied(String),
    Submitted(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerminalColor {
    Ansi16(u8),
    Ansi256(u8),
    Rgb(u8, u8, u8),
}

impl TerminalColor {
    fn foreground_escape(self) -> String {
        match self {
            Self::Ansi16(code) => ANSI_STYLES.color.ansi(code),
            Self::Ansi256(code) => ANSI_STYLES.color.ansi256(code),
            Self::Rgb(red, green, blue) => ANSI_STYLES.color.ansi16m(red, green, blue),
        }
    }

    fn background_escape(self) -> String {
        match self {
            Self::Ansi16(code) => ANSI_STYLES.bg_color.ansi(code),
            Self::Ansi256(code) => ANSI_STYLES.bg_color.ansi256(code),
            Self::Rgb(red, green, blue) => ANSI_STYLES.bg_color.ansi16m(red, green, blue),
        }
    }
}

fn command_color_escape() -> String {
    BLUE.open_escape()
}

fn selected_dropdown_color_escape() -> String {
    BLUE.open_escape()
}

fn unselected_dropdown_color() -> TerminalColor {
    TerminalColor::Ansi16(WHITE.open)
}

fn unselected_dropdown_description_color() -> TerminalColor {
    TerminalColor::Ansi16(GRAY.open)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputOption {
    pub command: String,
    pub description: String,
}

impl InputOption {
    pub fn new(command: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            command: normalize_command(command.into()),
            description: description.into().trim().to_string(),
        }
    }

    fn is_empty(&self) -> bool {
        self.command.is_empty()
    }
}

impl From<&str> for InputOption {
    fn from(value: &str) -> Self {
        Self::new(value, "")
    }
}

impl From<String> for InputOption {
    fn from(value: String) -> Self {
        Self::new(value, "")
    }
}

impl From<(&str, &str)> for InputOption {
    fn from((command, description): (&str, &str)) -> Self {
        Self::new(command, description)
    }
}

impl From<(String, String)> for InputOption {
    fn from((command, description): (String, String)) -> Self {
        Self::new(command, description)
    }
}

impl From<(String, &str)> for InputOption {
    fn from((command, description): (String, &str)) -> Self {
        Self::new(command, description)
    }
}

impl From<(&str, String)> for InputOption {
    fn from((command, description): (&str, String)) -> Self {
        Self::new(command, description)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InputTheme {
    pub border_color: Option<TerminalColor>,
    pub text_color: Option<TerminalColor>,
    pub background_color: Option<TerminalColor>,
    pub suggestion_color: Option<TerminalColor>,
    pub suggestion_background_color: Option<TerminalColor>,
    pub selected_text_color: Option<TerminalColor>,
    pub selected_background_color: Option<TerminalColor>,
}

impl InputTheme {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ocean() -> Self {
        Self::new()
            .with_border_color(TerminalColor::Ansi256(81))
            .with_text_color(TerminalColor::Ansi256(255))
            .with_background_color(TerminalColor::Ansi256(24))
            .with_suggestion_color(unselected_dropdown_color())
            .with_suggestion_background_color(TerminalColor::Ansi256(23))
            .with_selected_background_color(TerminalColor::Ansi256(31))
    }

    pub fn with_border_color(mut self, color: TerminalColor) -> Self {
        self.border_color = Some(color);
        self
    }

    pub fn with_text_color(mut self, color: TerminalColor) -> Self {
        self.text_color = Some(color);
        self
    }

    pub fn with_background_color(mut self, color: TerminalColor) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn with_suggestion_color(mut self, color: TerminalColor) -> Self {
        self.suggestion_color = Some(color);
        self
    }

    pub fn with_suggestion_background_color(mut self, color: TerminalColor) -> Self {
        self.suggestion_background_color = Some(color);
        self
    }

    pub fn with_selected_text_color(mut self, color: TerminalColor) -> Self {
        self.selected_text_color = Some(color);
        self
    }

    pub fn with_selected_background_color(mut self, color: TerminalColor) -> Self {
        self.selected_background_color = Some(color);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputView {
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_column: usize,
}

impl InputView {
    pub fn as_string(&self) -> String {
        self.lines.join("\n")
    }
}

impl fmt::Display for InputView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.as_string())
    }
}

#[derive(Debug, Clone)]
pub struct InputRenderer {
    rendered_rows: usize,
    cursor_row: usize,
    terminal_columns: usize,
}

impl InputRenderer {
    pub fn new(terminal_columns: usize) -> Self {
        Self {
            rendered_rows: 0,
            cursor_row: 0,
            terminal_columns: terminal_columns.max(1),
        }
    }

    pub fn render<W: Write>(&mut self, output: &mut W, input: &SlashInput) -> io::Result<()> {
        self.render_view(output, &input.render_with_terminal_width(self.terminal_columns))
    }

    pub fn render_view<W: Write>(&mut self, output: &mut W, view: &InputView) -> io::Result<()> {
        self.clear(output)?;

        for (index, line) in view.lines.iter().enumerate() {
            write!(output, "{line}")?;
            if index + 1 < view.lines.len() {
                output.write_all(b"\r\n")?;
            }
        }

        let rows = view
            .lines
            .iter()
            .map(|line| display_rows(line, self.terminal_columns))
            .collect::<Vec<_>>();
        let cursor_row_offset = view.cursor_column / self.terminal_columns;
        let cursor_column = view.cursor_column % self.terminal_columns;
        let absolute_cursor_display_row =
            rows.iter().take(view.cursor_row).sum::<usize>() + cursor_row_offset;
        let total_rows = rows.iter().sum::<usize>();
        let lines_up = total_rows.saturating_sub(absolute_cursor_display_row + 1);
        if lines_up > 0 {
            write!(output, "\x1B[{lines_up}A")?;
        }
        write!(output, "\r")?;
        if cursor_column > 0 {
            write!(output, "\x1B[{}C", cursor_column)?;
        }
        write!(output, "\x1B[?25h")?;
        output.flush()?;

        self.rendered_rows = total_rows;
        self.cursor_row = absolute_cursor_display_row;
        Ok(())
    }

    pub fn clear<W: Write>(&mut self, output: &mut W) -> io::Result<()> {
        if self.rendered_rows == 0 {
            return Ok(());
        }

        write!(output, "\r")?;
        if self.cursor_row > 0 {
            write!(output, "\x1B[{}A", self.cursor_row)?;
        }
        write!(output, "\x1B[0J\x1B[?25h")?;
        output.flush()?;

        self.rendered_rows = 0;
        self.cursor_row = 0;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlashInput {
    options: Vec<InputOption>,
    value: String,
    cursor: usize,
    input_width: Option<usize>,
    theme: InputTheme,
    header_lines: Vec<String>,
    dropdown_open: bool,
    selected_suggestion: usize,
}

impl SlashInput {
    pub fn new<I, S>(options: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<InputOption>,
    {
        Self {
            options: collect_options(options),
            value: String::new(),
            cursor: 0,
            input_width: None,
            theme: InputTheme::default(),
            header_lines: Vec::new(),
            dropdown_open: false,
            selected_suggestion: 0,
        }
    }

    pub fn with_min_width<I, S>(options: I, min_width: usize) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<InputOption>,
    {
        Self {
            options: collect_options(options),
            value: String::new(),
            cursor: 0,
            input_width: normalize_input_width(Some(min_width)),
            theme: InputTheme::default(),
            header_lines: Vec::new(),
            dropdown_open: false,
            selected_suggestion: 0,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn options(&self) -> &[InputOption] {
        &self.options
    }

    pub fn with_input_width(mut self, input_width: Option<usize>) -> Self {
        self.input_width = normalize_input_width(input_width);
        self
    }

    pub fn set_input_width(&mut self, input_width: Option<usize>) {
        self.input_width = normalize_input_width(input_width);
    }

    pub fn with_options<I, S>(mut self, options: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<InputOption>,
    {
        self.options = collect_options(options);
        self.refresh_dropdown(false);
        self
    }

    pub fn set_options<I, S>(&mut self, options: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<InputOption>,
    {
        self.options = collect_options(options);
        self.refresh_dropdown(false);
    }

    pub fn theme(&self) -> &InputTheme {
        &self.theme
    }

    pub fn header_lines(&self) -> &[String] {
        &self.header_lines
    }

    pub fn with_header_lines<I, S>(mut self, lines: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.header_lines = collect_lines(lines);
        self
    }

    pub fn set_header_lines<I, S>(&mut self, lines: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.header_lines = collect_lines(lines);
    }

    pub fn with_theme(mut self, theme: InputTheme) -> Self {
        self.theme = theme;
        self
    }

    pub fn set_theme(&mut self, theme: InputTheme) {
        self.theme = theme;
    }

    pub fn with_background_color(mut self, color: TerminalColor) -> Self {
        self.theme = self.theme.clone().with_background_color(color);
        self
    }

    pub fn set_background_color(&mut self, color: TerminalColor) {
        self.theme.background_color = Some(color);
    }

    pub fn is_dropdown_visible(&self) -> bool {
        !self.filtered_options().is_empty()
    }

    pub fn selected_option(&self) -> Option<&InputOption> {
        let options = self.filtered_options();
        options.get(self.selected_suggestion).copied()
    }

    pub fn selected_command(&self) -> Option<&str> {
        self.selected_option().map(|option| option.command.as_str())
    }

    pub fn filtered_options(&self) -> Vec<&InputOption> {
        if !self.dropdown_open || !self.value.starts_with('/') {
            return Vec::new();
        }

        let query = self.value[1..].to_ascii_lowercase();

        self.options
            .iter()
            .filter(|option| {
                query.is_empty() || option.command.to_ascii_lowercase().starts_with(&query)
            })
            .collect()
    }

    pub fn filtered_commands(&self) -> Vec<&str> {
        self.filtered_options()
            .into_iter()
            .map(|option| option.command.as_str())
            .collect()
    }

    pub fn handle_key(&mut self, event: KeyEvent) -> InputAction {
        if let Some(action) = self.handle_shortcut(event) {
            return action;
        }

        match event.code {
            KeyCode::Char(ch) if self.is_text_input(event.modifiers) => {
                self.insert_char(ch);
                self.refresh_dropdown(true);
                InputAction::None
            }
            KeyCode::Backspace => {
                self.delete_backward();
                self.refresh_dropdown(true);
                InputAction::None
            }
            KeyCode::Delete => {
                self.delete_forward();
                self.refresh_dropdown(true);
                InputAction::None
            }
            KeyCode::Left => {
                self.cursor = self.cursor.saturating_sub(1);
                InputAction::None
            }
            KeyCode::Right => {
                self.cursor = (self.cursor + 1).min(self.char_len());
                InputAction::None
            }
            KeyCode::Home => {
                self.cursor = 0;
                InputAction::None
            }
            KeyCode::End => {
                self.cursor = self.char_len();
                InputAction::None
            }
            KeyCode::Up => {
                self.move_selection_up();
                InputAction::None
            }
            KeyCode::Down => {
                self.move_selection_down();
                InputAction::None
            }
            KeyCode::Esc => {
                self.dropdown_open = false;
                InputAction::None
            }
            KeyCode::Tab | KeyCode::Enter if self.is_dropdown_visible() => {
                self.apply_selected_command()
            }
            KeyCode::Enter => {
                let submitted = self.value.clone();
                self.clear();
                InputAction::Submitted(submitted)
            }
            _ => InputAction::None,
        }
    }

    pub fn handle_paste(&mut self, text: impl AsRef<str>) {
        self.insert_str(text.as_ref());
        self.refresh_dropdown(true);
    }

    pub fn render(&self) -> InputView {
        self.render_internal(None)
    }

    pub fn render_with_terminal_width(&self, terminal_columns: usize) -> InputView {
        self.render_internal(Some(terminal_columns))
    }

    fn render_internal(&self, terminal_columns: Option<usize>) -> InputView {
        let filtered = self.filtered_options();
        let (visible_start, visible_end) = self.visible_option_bounds(filtered.len());
        let visible_options = &filtered[visible_start..visible_end];
        let width = self.resolve_render_width(terminal_columns);
        let max_command_width = visible_options
            .iter()
            .map(|option| option.command.chars().count())
            .max()
            .unwrap_or(0);

        let mut lines = self.header_lines.clone();
        let border = self.render_container_fill(width);
        lines.push(border.clone());
        lines.push(self.render_input_line(width));
        lines.push(border);

        if !visible_options.is_empty() {
            lines.extend(std::iter::repeat_n(String::new(), DROPDOWN_TOP_PADDING));
            for (visible_index, option) in visible_options.iter().enumerate() {
                let index = visible_start + visible_index;
                let is_selected = index == self.selected_suggestion;
                let foreground = if is_selected {
                    self.theme.selected_text_color
                } else {
                    self.theme
                        .suggestion_color
                        .or(Some(unselected_dropdown_color()))
                };

                lines.push(self.render_dropdown_line(
                    option,
                    max_command_width,
                    foreground,
                    is_selected,
                ));
                if visible_index + 1 < visible_options.len() {
                    lines.extend(std::iter::repeat_n(
                        String::new(),
                        DROPDOWN_VERTICAL_SPACING,
                    ));
                }
            }
        }

        InputView {
            lines,
            cursor_row: self.header_lines.len() + 1,
            cursor_column: 1 + self.cursor,
        }
    }

    fn resolve_render_width(&self, terminal_columns: Option<usize>) -> usize {
        let content_width = self.char_len().max(1);

        match self.input_width {
            Some(width) => width.max(content_width),
            None => terminal_columns
                .map(content_width_from_terminal_columns)
                .unwrap_or(DEFAULT_INPUT_WIDTH)
                .max(content_width),
        }
    }

    fn visible_option_bounds(&self, filtered_len: usize) -> (usize, usize) {
        if filtered_len == 0 {
            return (0, 0);
        }

        if filtered_len <= MAX_VISIBLE_DROPDOWN_OPTIONS {
            return (0, filtered_len);
        }

        let selected = self.selected_suggestion.min(filtered_len - 1);
        let max_start = filtered_len - MAX_VISIBLE_DROPDOWN_OPTIONS;
        let start = selected
            .saturating_add(1)
            .saturating_sub(MAX_VISIBLE_DROPDOWN_OPTIONS)
            .min(max_start);

        (start, start + MAX_VISIBLE_DROPDOWN_OPTIONS)
    }

    fn render_container_fill(&self, width: usize) -> String {
        style_text(
            &" ".repeat(width + 2),
            None,
            self.container_background_color(),
        )
    }

    fn container_background_color(&self) -> Option<TerminalColor> {
        self.theme.background_color.or(self.theme.border_color)
    }

    fn render_dropdown_line(
        &self,
        option: &InputOption,
        max_command_width: usize,
        foreground: Option<TerminalColor>,
        is_selected: bool,
    ) -> String {
        let command_segment = format!(" /{}", option.command);
        let description_segment = if option.description.is_empty() {
            None
        } else {
            let command_width = option.command.chars().count();
            let spacing = 2 + max_command_width.saturating_sub(command_width);
            Some(format!("{}{}", " ".repeat(spacing), option.description))
        };

        let mut line = command_segment.clone();
        if let Some(description_segment) = &description_segment {
            line.push_str(description_segment);
        }

        if is_selected && foreground.is_none() {
            return style_text_with_escape(&line, Some(selected_dropdown_color_escape()), None);
        }

        if is_selected {
            return style_text(&line, foreground, None);
        }

        let mut rendered = style_text(&command_segment, foreground, None);
        if let Some(description_segment) = description_segment {
            rendered.push_str(&style_text(
                &description_segment,
                Some(unselected_dropdown_description_color()),
                None,
            ));
        }

        rendered
    }

    fn render_input_line(&self, width: usize) -> String {
        let background = self.container_background_color();
        let left = style_text(" ", None, background);
        let body = self.render_input_body(width);
        let right = style_text(" ", None, background);
        format!("{left}{body}{right}")
    }

    fn render_input_body(&self, width: usize) -> String {
        let background = self.container_background_color();

        if let Some((command, remainder)) = split_applied_command(&self.value) {
            let mut body = String::new();
            body.push_str(&style_text_with_escape(
                command,
                Some(command_color_escape()),
                background,
            ));
            body.push_str(&style_text(
                remainder,
                self.theme.text_color,
                background,
            ));

            let visible_len = command.chars().count() + remainder.chars().count();
            if visible_len < width {
                body.push_str(&style_text(
                    &" ".repeat(width - visible_len),
                    self.theme.text_color,
                    background,
                ));
            }

            body
        } else {
            style_text(
                &format!(
                    "{}{}",
                    self.value,
                    " ".repeat(width - self.value.chars().count())
                ),
                self.theme.text_color,
                background,
            )
        }
    }

    fn handle_shortcut(&self, event: KeyEvent) -> Option<InputAction> {
        if !event
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::SUPER)
        {
            return None;
        }

        match event.code {
            KeyCode::Char(ch) if ch.eq_ignore_ascii_case(&'c') => {
                Some(InputAction::CopyRequested(self.value.clone()))
            }
            KeyCode::Char(ch) if ch.eq_ignore_ascii_case(&'v') => Some(InputAction::PasteRequested),
            _ => Some(InputAction::None),
        }
    }

    fn is_text_input(&self, modifiers: KeyModifiers) -> bool {
        !modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER)
    }

    fn insert_char(&mut self, ch: char) {
        let byte_index = char_to_byte_index(&self.value, self.cursor);
        self.value.insert(byte_index, ch);
        self.cursor += 1;
    }

    fn insert_str(&mut self, text: &str) {
        let byte_index = char_to_byte_index(&self.value, self.cursor);
        self.value.insert_str(byte_index, text);
        self.cursor += text.chars().count();
    }

    fn delete_backward(&mut self) {
        if self.cursor == 0 {
            return;
        }

        let start = char_to_byte_index(&self.value, self.cursor - 1);
        let end = char_to_byte_index(&self.value, self.cursor);
        self.value.replace_range(start..end, "");
        self.cursor -= 1;
    }

    fn delete_forward(&mut self) {
        if self.cursor >= self.char_len() {
            return;
        }

        let start = char_to_byte_index(&self.value, self.cursor);
        let end = char_to_byte_index(&self.value, self.cursor + 1);
        self.value.replace_range(start..end, "");
    }

    fn move_selection_up(&mut self) {
        let count = self.filtered_options().len();
        if count == 0 {
            return;
        }

        self.selected_suggestion = if self.selected_suggestion == 0 {
            count - 1
        } else {
            self.selected_suggestion - 1
        };
    }

    fn move_selection_down(&mut self) {
        let count = self.filtered_options().len();
        if count == 0 {
            return;
        }

        self.selected_suggestion = if self.selected_suggestion + 1 >= count {
            0
        } else {
            self.selected_suggestion + 1
        };
    }

    fn apply_selected_command(&mut self) -> InputAction {
        let Some(command) = self.selected_command().map(str::to_owned) else {
            return InputAction::None;
        };

        self.value = format!("/{command} ");
        self.cursor = self.char_len();
        self.dropdown_open = false;
        self.selected_suggestion = 0;
        InputAction::SuggestionApplied(self.value.clone())
    }

    fn refresh_dropdown(&mut self, reset_selection: bool) {
        self.dropdown_open = self.value.starts_with('/');

        if !self.dropdown_open {
            self.selected_suggestion = 0;
            return;
        }

        let count = self.filtered_options().len();
        if count == 0 || reset_selection {
            self.selected_suggestion = 0;
        } else {
            self.selected_suggestion = self.selected_suggestion.min(count - 1);
        }
    }

    fn char_len(&self) -> usize {
        self.value.chars().count()
    }

    fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
        self.dropdown_open = false;
        self.selected_suggestion = 0;
    }
}

fn normalize_command(command: String) -> String {
    command.trim().trim_start_matches('/').to_string()
}

fn normalize_input_width(input_width: Option<usize>) -> Option<usize> {
    input_width.map(|width| width.max(1))
}

fn content_width_from_terminal_columns(terminal_columns: usize) -> usize {
    terminal_columns.saturating_sub(2).max(1)
}

fn collect_options<I, S>(options: I) -> Vec<InputOption>
where
    I: IntoIterator<Item = S>,
    S: Into<InputOption>,
{
    options
        .into_iter()
        .map(Into::into)
        .filter(|option| !option.is_empty())
        .collect()
}

fn collect_lines<I, S>(lines: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    lines.into_iter().map(Into::into).collect()
}

fn display_rows(line: &str, terminal_columns: usize) -> usize {
    let width = visible_text_width(line);
    width.max(1).div_ceil(terminal_columns.max(1))
}

fn visible_text_width(text: &str) -> usize {
    let mut visible_width = 0;
    let mut chars = text.chars();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if matches!(chars.next(), Some('[')) {
                for next in chars.by_ref() {
                    if ('@'..='~').contains(&next) {
                        break;
                    }
                }
            }
            continue;
        }

        if !matches!(ch, '\n' | '\r') && !ch.is_control() {
            visible_width += 1;
        }
    }

    visible_width
}

fn split_applied_command(value: &str) -> Option<(&str, &str)> {
    if !value.starts_with('/') {
        return None;
    }

    let split_at = value.find(' ')?;
    if split_at == 1 {
        return None;
    }

    Some(value.split_at(split_at))
}

fn char_to_byte_index(input: &str, char_index: usize) -> usize {
    input
        .char_indices()
        .nth(char_index)
        .map(|(index, _)| index)
        .unwrap_or(input.len())
}

fn style_text(
    text: &str,
    foreground: Option<TerminalColor>,
    background: Option<TerminalColor>,
) -> String {
    style_text_with_escape(
        text,
        foreground.map(TerminalColor::foreground_escape),
        background,
    )
}

fn style_text_with_escape(
    text: &str,
    foreground_escape: Option<String>,
    background: Option<TerminalColor>,
) -> String {
    if foreground_escape.is_none() && background.is_none() {
        return text.to_string();
    }

    let mut styled = String::new();

    let has_foreground = foreground_escape.is_some();

    if let Some(foreground_escape) = foreground_escape {
        styled.push_str(&foreground_escape);
    }
    if let Some(background) = background {
        styled.push_str(&background.background_escape());
    }

    styled.push_str(text);

    if background.is_some() {
        styled.push_str(ANSI_STYLES.bg_color.close);
    }
    if has_foreground {
        styled.push_str(ANSI_STYLES.color.close);
    }

    styled
}

#[cfg(test)]
mod tests {
    use super::*;

    fn type_text(input: &mut SlashInput, text: &str) {
        for ch in text.chars() {
            input.handle_key(KeyEvent::plain(KeyCode::Char(ch)));
        }
    }

    #[test]
    fn renders_input_box_and_accepts_text() {
        let mut input = SlashInput::new(["help", "exit"]);
        type_text(&mut input, "hello");

        let width = DEFAULT_INPUT_WIDTH.max("hello".chars().count());
        let border = " ".repeat(width + 2);
        let line = format!(" hello{} ", " ".repeat(width - "hello".chars().count()));

        assert_eq!(input.value(), "hello");
        assert_eq!(input.render().lines, vec![border.clone(), line, border]);
    }

    #[test]
    fn reports_copy_and_paste_shortcuts() {
        let mut input = SlashInput::new(["help"]);
        type_text(&mut input, "hello");

        assert_eq!(
            input.handle_key(KeyEvent::ctrl(KeyCode::Char('c'))),
            InputAction::CopyRequested("hello".to_string())
        );
        assert_eq!(
            input.handle_key(KeyEvent::super_key(KeyCode::Char('v'))),
            InputAction::PasteRequested
        );

        input.handle_paste(" world");
        assert_eq!(input.value(), "hello world");
    }

    #[test]
    fn leading_slash_opens_filtered_dropdown() {
        let mut input = SlashInput::new(["help", "hello", "exit"]);
        type_text(&mut input, "/he");

        assert!(input.is_dropdown_visible());
        assert_eq!(input.filtered_commands(), vec!["help", "hello"]);

        let rendered = input.render().as_string();
        assert!(rendered.contains(" /help"));
        assert!(rendered.contains(" /hello"));
        assert!(!rendered.contains("/exit"));
    }

    #[test]
    fn slash_only_works_for_the_first_character() {
        let mut input = SlashInput::new(["help"]);
        type_text(&mut input, "a/");

        assert!(!input.is_dropdown_visible());
    }

    #[test]
    fn arrow_keys_change_selection_and_enter_applies_it() {
        let mut input = SlashInput::new(["help", "hello", "history"]);
        type_text(&mut input, "/h");

        input.handle_key(KeyEvent::plain(KeyCode::Down));
        input.handle_key(KeyEvent::plain(KeyCode::Down));

        assert_eq!(input.selected_command(), Some("history"));
        assert_eq!(
            input.handle_key(KeyEvent::plain(KeyCode::Enter)),
            InputAction::SuggestionApplied("/history ".to_string())
        );
        assert_eq!(input.value(), "/history ");
        assert!(!input.is_dropdown_visible());
    }

    #[test]
    fn enter_submits_when_dropdown_is_not_visible() {
        let mut input = SlashInput::new(["help"]);
        type_text(&mut input, "run");

        assert_eq!(
            input.handle_key(KeyEvent::plain(KeyCode::Enter)),
            InputAction::Submitted("run".to_string())
        );
        assert_eq!(input.value(), "");
        assert_eq!(input.cursor(), 0);
        assert!(!input.is_dropdown_visible());
    }

    #[test]
    fn render_supports_custom_background_color() {
        let mut input = SlashInput::new(["help"]);
        input.set_background_color(TerminalColor::Ansi256(236));

        let rendered = input.render().as_string();
        assert!(rendered.contains("\x1B[48;5;236m"));
    }

    #[test]
    fn container_fill_uses_background_color() {
        let input = SlashInput::new(["help"]).with_theme(
            InputTheme::ocean()
                .with_border_color(TerminalColor::Ansi256(81))
                .with_background_color(TerminalColor::Ansi256(236)),
        );

        let view = input.render();
        assert!(view.lines[0].starts_with("\x1B[48;5;236m "));
        assert!(view.lines[1].starts_with("\x1B[48;5;236m "));
    }

    #[test]
    fn input_width_can_be_overridden_or_reset_to_default() {
        let mut input = SlashInput::new(["help"]).with_input_width(Some(12));
        type_text(&mut input, "hello");

        assert_eq!(input.render().lines[0], " ".repeat(14));

        input.set_input_width(None);

        let width = DEFAULT_INPUT_WIDTH.max("hello".chars().count());
        let border = " ".repeat(width + 2);
        assert_eq!(input.render().lines[0], border);
    }

    #[test]
    fn theme_builder_applies_selected_suggestion_colors() {
        let input = SlashInput::new(["help", "hello"]).with_theme(
            InputTheme::ocean().with_selected_background_color(TerminalColor::Rgb(10, 20, 30)),
        );

        let themed = input.with_background_color(TerminalColor::Ansi16(40));
        assert_eq!(
            themed.theme().background_color,
            Some(TerminalColor::Ansi16(40))
        );
        assert_eq!(
            themed.theme().selected_background_color,
            Some(TerminalColor::Rgb(10, 20, 30))
        );
    }

    #[test]
    fn options_can_be_replaced_from_outside() {
        let mut input = SlashInput::new(["help", "hello"]);
        type_text(&mut input, "/h");

        assert_eq!(input.filtered_commands(), vec!["help", "hello"]);

        input.set_options(["history", "theme"]);

        assert_eq!(
            input.options(),
            &[
                InputOption::new("history", ""),
                InputOption::new("theme", "")
            ]
        );
        assert_eq!(input.filtered_commands(), vec!["history"]);
        assert_eq!(
            input.selected_option(),
            Some(&InputOption::new("history", ""))
        );
    }

    #[test]
    fn with_options_replaces_existing_dropdown_data() {
        let input = SlashInput::new(["help"]).with_options(["clear", "/quit"]);

        assert_eq!(
            input.options(),
            &[InputOption::new("clear", ""), InputOption::new("quit", "")]
        );
    }

    #[test]
    fn applied_command_is_rendered_in_blue_with_a_trailing_space() {
        let mut input = SlashInput::new(["help", "history"]);
        type_text(&mut input, "/h");

        input.handle_key(KeyEvent::plain(KeyCode::Down));
        input.handle_key(KeyEvent::plain(KeyCode::Enter));

        let rendered = input.render().as_string();
        assert_eq!(input.value(), "/history ");
        assert!(rendered.contains("\x1B[34m/history"));
        assert!(rendered.contains("\x1B[39m "));
    }

    #[test]
    fn dropdown_render_has_no_marker_border_or_background() {
        let mut input = SlashInput::new(["help", "history"]).with_theme(
            InputTheme::ocean()
                .with_suggestion_background_color(TerminalColor::Ansi256(238))
                .with_selected_background_color(TerminalColor::Ansi256(31)),
        );
        type_text(&mut input, "/h");

        let view = input.render();
        let rendered = view.as_string();
        assert!(!rendered.contains("> /"));
        assert!(!view.lines[3 + DROPDOWN_TOP_PADDING].starts_with("| "));
        assert!(!rendered.contains("\x1B[48;5;238m"));
        assert!(!rendered.contains("\x1B[48;5;31m"));
    }

    #[test]
    fn dropdown_renders_command_and_description_with_more_spacing() {
        let mut input = SlashInput::new([
            ("help", "Show available commands"),
            ("history", "Previous commands"),
        ]);
        type_text(&mut input, "/h");

        let view = input.render();
        assert_eq!(view.lines.len(), 3 + DROPDOWN_TOP_PADDING + 2);
        assert!(view.lines[3 + DROPDOWN_TOP_PADDING].contains(" /help     Show available commands"));
        assert!(view.lines[4 + DROPDOWN_TOP_PADDING].contains(" /history"));
        assert!(view.lines[4 + DROPDOWN_TOP_PADDING].contains("  Previous commands"));
    }

    #[test]
    fn dropdown_uses_white_for_unselected_and_blue_for_selected() {
        let mut input = SlashInput::new([
            ("help", "Show available commands"),
            ("history", "Previous commands"),
        ]);
        type_text(&mut input, "/h");

        let rendered = input.render().as_string();
        assert!(rendered.contains("\x1B[34m /help     Show available commands"));
        assert!(rendered.contains("\x1B[37m /history\x1B[39m\x1B[90m  Previous commands\x1B[39m"));
    }

    #[test]
    fn dropdown_renders_at_most_eight_visible_options() {
        let options = (1..=10).map(|index| {
            (
                format!("help{index}"),
                format!("Show available command {index}"),
            )
        });
        let mut input = SlashInput::new(options);
        type_text(&mut input, "/h");

        let view = input.render();
        let rendered = view.as_string();

        assert_eq!(view.lines.len(), 3 + DROPDOWN_TOP_PADDING + 8);
        assert!(rendered.contains(" /help1"));
        assert!(rendered.contains(" /help8"));
        assert!(!rendered.contains(" /help9"));
        assert!(!rendered.contains(" /help10"));
    }

    #[test]
    fn dropdown_scrolls_when_selection_moves_past_visible_window() {
        let options = (1..=10).map(|index| {
            (
                format!("help{index}"),
                format!("Show available command {index}"),
            )
        });
        let mut input = SlashInput::new(options);
        type_text(&mut input, "/h");

        for _ in 0..8 {
            input.handle_key(KeyEvent::plain(KeyCode::Down));
        }

        let rendered = input.render().as_string();

        assert_eq!(input.selected_command(), Some("help9"));
        assert!(!rendered.contains(" /help1"));
        assert!(rendered.contains(" /help2"));
        assert!(rendered.contains("\x1B[34m /help9"));
        assert!(!rendered.contains(" /help10"));
    }

    #[test]
    fn dropdown_selection_wraps_between_first_and_last_options() {
        let options = (1..=10).map(|index| {
            (
                format!("help{index}"),
                format!("Show available command {index}"),
            )
        });
        let mut input = SlashInput::new(options);
        type_text(&mut input, "/h");

        input.handle_key(KeyEvent::plain(KeyCode::Up));
        assert_eq!(input.selected_command(), Some("help10"));

        let wrapped_to_last = input.render();
        assert!(wrapped_to_last.lines[3 + DROPDOWN_TOP_PADDING].contains(" /help3"));
        assert!(wrapped_to_last.lines.last().is_some_and(|line| line.contains("\x1B[34m /help10")));

        input.handle_key(KeyEvent::plain(KeyCode::Down));
        assert_eq!(input.selected_command(), Some("help1"));

        let wrapped_to_first = input.render();
        assert!(wrapped_to_first.lines[3 + DROPDOWN_TOP_PADDING].contains("\x1B[34m /help1"));
        assert!(wrapped_to_first.lines.last().is_some_and(|line| line.contains(" /help8")));
    }

    #[test]
    fn render_with_terminal_width_uses_available_columns() {
        let mut input = SlashInput::new(["help"]);
        type_text(&mut input, "hello");

        let view = input.render_with_terminal_width(40);
        let width = content_width_from_terminal_columns(40);
        let border = " ".repeat(width + 2);
        let line = format!(" hello{} ", " ".repeat(width - "hello".chars().count()));

        assert_eq!(view.lines[0], border);
        assert_eq!(view.lines[1], line);
        assert_eq!(view.lines[2], border);
    }
}
