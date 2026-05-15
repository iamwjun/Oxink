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
            .map(|option| text_display_width(&option.command))
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
            cursor_column: 1 + self.cursor_display_width(),
        }
    }

    fn resolve_render_width(&self, terminal_columns: Option<usize>) -> usize {
        let content_width = self.input_display_width().max(1);

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
            let command_width = text_display_width(&option.command);
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

            let visible_len = text_display_width(command) + text_display_width(remainder);
            if visible_len < width {
                body.push_str(&style_text(
                    &" ".repeat(width - visible_len),
                    self.theme.text_color,
                    background,
                ));
            }

            body
        } else {
            let visible_len = self.input_display_width();
            style_text(
                &format!(
                    "{}{}",
                    self.value,
                    " ".repeat(width.saturating_sub(visible_len))
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

    fn input_display_width(&self) -> usize {
        text_display_width(&self.value)
    }

    fn cursor_display_width(&self) -> usize {
        text_display_width_up_to(&self.value, self.cursor)
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

fn text_display_width(text: &str) -> usize {
    text.chars().map(char_display_width).sum()
}

fn text_display_width_up_to(text: &str, char_count: usize) -> usize {
    text.chars().take(char_count).map(char_display_width).sum()
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
            visible_width += char_display_width(ch);
        }
    }

    visible_width
}

fn char_display_width(ch: char) -> usize {
    if ch.is_control() || is_zero_width_char(ch) {
        return 0;
    }

    if is_wide_char(ch) {
        2
    } else {
        1
    }
}

fn is_zero_width_char(ch: char) -> bool {
    matches!(
        ch,
        '\u{0300}'..='\u{036F}'
            | '\u{0483}'..='\u{0489}'
            | '\u{0591}'..='\u{05BD}'
            | '\u{05BF}'
            | '\u{05C1}'..='\u{05C2}'
            | '\u{05C4}'..='\u{05C5}'
            | '\u{05C7}'
            | '\u{0610}'..='\u{061A}'
            | '\u{064B}'..='\u{065F}'
            | '\u{0670}'
            | '\u{06D6}'..='\u{06DC}'
            | '\u{06DF}'..='\u{06E4}'
            | '\u{06E7}'..='\u{06E8}'
            | '\u{06EA}'..='\u{06ED}'
            | '\u{0711}'
            | '\u{0730}'..='\u{074A}'
            | '\u{07A6}'..='\u{07B0}'
            | '\u{07EB}'..='\u{07F3}'
            | '\u{0816}'..='\u{0819}'
            | '\u{081B}'..='\u{0823}'
            | '\u{0825}'..='\u{0827}'
            | '\u{0829}'..='\u{082D}'
            | '\u{0859}'..='\u{085B}'
            | '\u{08D3}'..='\u{08E1}'
            | '\u{08E3}'..='\u{0902}'
            | '\u{093A}'
            | '\u{093C}'
            | '\u{0941}'..='\u{0948}'
            | '\u{094D}'
            | '\u{0951}'..='\u{0957}'
            | '\u{0962}'..='\u{0963}'
            | '\u{0981}'
            | '\u{09BC}'
            | '\u{09C1}'..='\u{09C4}'
            | '\u{09CD}'
            | '\u{09E2}'..='\u{09E3}'
            | '\u{0A01}'..='\u{0A02}'
            | '\u{0A3C}'
            | '\u{0A41}'..='\u{0A42}'
            | '\u{0A47}'..='\u{0A48}'
            | '\u{0A4B}'..='\u{0A4D}'
            | '\u{0A51}'
            | '\u{0A70}'..='\u{0A71}'
            | '\u{0A75}'
            | '\u{0A81}'..='\u{0A82}'
            | '\u{0ABC}'
            | '\u{0AC1}'..='\u{0AC5}'
            | '\u{0AC7}'..='\u{0AC8}'
            | '\u{0ACD}'
            | '\u{0AE2}'..='\u{0AE3}'
            | '\u{0B01}'
            | '\u{0B3C}'
            | '\u{0B3F}'
            | '\u{0B41}'..='\u{0B44}'
            | '\u{0B4D}'
            | '\u{0B56}'
            | '\u{0B62}'..='\u{0B63}'
            | '\u{0B82}'
            | '\u{0BC0}'
            | '\u{0BCD}'
            | '\u{0C00}'
            | '\u{0C04}'
            | '\u{0C3E}'..='\u{0C40}'
            | '\u{0C46}'..='\u{0C48}'
            | '\u{0C4A}'..='\u{0C4D}'
            | '\u{0C55}'..='\u{0C56}'
            | '\u{0C62}'..='\u{0C63}'
            | '\u{0C81}'
            | '\u{0CBC}'
            | '\u{0CBF}'
            | '\u{0CC6}'
            | '\u{0CCC}'..='\u{0CCD}'
            | '\u{0CE2}'..='\u{0CE3}'
            | '\u{0D00}'..='\u{0D01}'
            | '\u{0D3B}'..='\u{0D3C}'
            | '\u{0D41}'..='\u{0D44}'
            | '\u{0D4D}'
            | '\u{0D62}'..='\u{0D63}'
            | '\u{0DCA}'
            | '\u{0DD2}'..='\u{0DD4}'
            | '\u{0DD6}'
            | '\u{0E31}'
            | '\u{0E34}'..='\u{0E3A}'
            | '\u{0E47}'..='\u{0E4E}'
            | '\u{0EB1}'
            | '\u{0EB4}'..='\u{0EBC}'
            | '\u{0EC8}'..='\u{0ECE}'
            | '\u{0F18}'..='\u{0F19}'
            | '\u{0F35}'
            | '\u{0F37}'
            | '\u{0F39}'
            | '\u{0F71}'..='\u{0F7E}'
            | '\u{0F80}'..='\u{0F84}'
            | '\u{0F86}'..='\u{0F87}'
            | '\u{0F8D}'..='\u{0F97}'
            | '\u{0F99}'..='\u{0FBC}'
            | '\u{0FC6}'
            | '\u{102D}'..='\u{1030}'
            | '\u{1032}'..='\u{1037}'
            | '\u{1039}'..='\u{103A}'
            | '\u{103D}'..='\u{103E}'
            | '\u{1058}'..='\u{1059}'
            | '\u{105E}'..='\u{1060}'
            | '\u{1071}'..='\u{1074}'
            | '\u{1082}'
            | '\u{1085}'..='\u{1086}'
            | '\u{108D}'
            | '\u{109D}'
            | '\u{135D}'..='\u{135F}'
            | '\u{1712}'..='\u{1714}'
            | '\u{1732}'..='\u{1734}'
            | '\u{1752}'..='\u{1753}'
            | '\u{1772}'..='\u{1773}'
            | '\u{17B4}'..='\u{17B5}'
            | '\u{17B7}'..='\u{17BD}'
            | '\u{17C6}'
            | '\u{17C9}'..='\u{17D3}'
            | '\u{17DD}'
            | '\u{180B}'..='\u{180F}'
            | '\u{1885}'..='\u{1886}'
            | '\u{18A9}'
            | '\u{1920}'..='\u{1922}'
            | '\u{1927}'..='\u{1928}'
            | '\u{1932}'
            | '\u{1939}'..='\u{193B}'
            | '\u{1A17}'..='\u{1A18}'
            | '\u{1A1B}'
            | '\u{1A56}'
            | '\u{1A58}'..='\u{1A5E}'
            | '\u{1A60}'
            | '\u{1A62}'
            | '\u{1A65}'..='\u{1A6C}'
            | '\u{1A73}'..='\u{1A7C}'
            | '\u{1A7F}'
            | '\u{1AB0}'..='\u{1ACE}'
            | '\u{1B00}'..='\u{1B03}'
            | '\u{1B34}'
            | '\u{1B36}'..='\u{1B3A}'
            | '\u{1B3C}'
            | '\u{1B42}'
            | '\u{1B6B}'..='\u{1B73}'
            | '\u{1B80}'..='\u{1B81}'
            | '\u{1BA2}'..='\u{1BA5}'
            | '\u{1BA8}'..='\u{1BA9}'
            | '\u{1BAB}'..='\u{1BAD}'
            | '\u{1BE6}'
            | '\u{1BE8}'..='\u{1BE9}'
            | '\u{1BED}'
            | '\u{1BEF}'..='\u{1BF1}'
            | '\u{1C2C}'..='\u{1C33}'
            | '\u{1C36}'..='\u{1C37}'
            | '\u{1CD0}'..='\u{1CD2}'
            | '\u{1CD4}'..='\u{1CE0}'
            | '\u{1CE2}'..='\u{1CE8}'
            | '\u{1CED}'
            | '\u{1CF4}'
            | '\u{1CF8}'..='\u{1CF9}'
            | '\u{1DC0}'..='\u{1DFF}'
            | '\u{200B}'..='\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{206F}'
            | '\u{20D0}'..='\u{20F0}'
            | '\u{2CEF}'..='\u{2CF1}'
            | '\u{2D7F}'
            | '\u{2DE0}'..='\u{2DFF}'
            | '\u{302A}'..='\u{302F}'
            | '\u{3099}'..='\u{309A}'
            | '\u{A66F}'
            | '\u{A674}'..='\u{A67D}'
            | '\u{A69E}'..='\u{A69F}'
            | '\u{A6F0}'..='\u{A6F1}'
            | '\u{A802}'
            | '\u{A806}'
            | '\u{A80B}'
            | '\u{A825}'..='\u{A826}'
            | '\u{A82C}'
            | '\u{A8C4}'..='\u{A8C5}'
            | '\u{A8E0}'..='\u{A8F1}'
            | '\u{A8FF}'..='\u{A901}'
            | '\u{A926}'..='\u{A92D}'
            | '\u{A947}'..='\u{A951}'
            | '\u{A980}'..='\u{A982}'
            | '\u{A9B3}'
            | '\u{A9B6}'..='\u{A9B9}'
            | '\u{A9BC}'
            | '\u{A9E5}'
            | '\u{AA29}'..='\u{AA2E}'
            | '\u{AA31}'..='\u{AA32}'
            | '\u{AA35}'..='\u{AA36}'
            | '\u{AA43}'
            | '\u{AA4C}'
            | '\u{AA7C}'
            | '\u{AAB0}'
            | '\u{AAB2}'..='\u{AAB4}'
            | '\u{AAB7}'..='\u{AAB8}'
            | '\u{AABE}'..='\u{AABF}'
            | '\u{AAC1}'
            | '\u{AAEC}'..='\u{AAED}'
            | '\u{AAF6}'
            | '\u{ABE5}'
            | '\u{ABE8}'
            | '\u{ABED}'
            | '\u{FB1E}'
            | '\u{FE00}'..='\u{FE0F}'
            | '\u{FE20}'..='\u{FE2F}'
            | '\u{FEFF}'
            | '\u{FFF9}'..='\u{FFFB}'
            | '\u{101FD}'
            | '\u{102E0}'
            | '\u{10376}'..='\u{1037A}'
            | '\u{10A01}'..='\u{10A03}'
            | '\u{10A05}'..='\u{10A06}'
            | '\u{10A0C}'..='\u{10A0F}'
            | '\u{10A38}'..='\u{10A3A}'
            | '\u{10A3F}'
            | '\u{10AE5}'..='\u{10AE6}'
            | '\u{11000}'..='\u{11002}'
            | '\u{11038}'..='\u{11046}'
            | '\u{1107F}'..='\u{11082}'
            | '\u{110B3}'..='\u{110B6}'
            | '\u{110B9}'..='\u{110BA}'
            | '\u{11100}'..='\u{11102}'
            | '\u{11127}'..='\u{1112B}'
            | '\u{1112D}'..='\u{11134}'
            | '\u{11173}'
            | '\u{11180}'..='\u{11181}'
            | '\u{111B6}'..='\u{111BE}'
            | '\u{111C9}'..='\u{111CC}'
            | '\u{1122F}'..='\u{11231}'
            | '\u{11234}'
            | '\u{11236}'..='\u{11237}'
            | '\u{1123E}'
            | '\u{112DF}'
            | '\u{112E3}'..='\u{112EA}'
            | '\u{11300}'..='\u{11301}'
            | '\u{1133C}'
            | '\u{11340}'
            | '\u{11366}'..='\u{1136C}'
            | '\u{11370}'..='\u{11374}'
            | '\u{11438}'..='\u{1143F}'
            | '\u{11442}'..='\u{11444}'
            | '\u{11446}'
            | '\u{114B3}'..='\u{114B8}'
            | '\u{114BA}'
            | '\u{114BF}'..='\u{114C0}'
            | '\u{114C2}'..='\u{114C3}'
            | '\u{115B2}'..='\u{115B5}'
            | '\u{115BC}'..='\u{115BD}'
            | '\u{115BF}'..='\u{115C0}'
            | '\u{115DC}'..='\u{115DD}'
            | '\u{11633}'..='\u{1163A}'
            | '\u{1163D}'
            | '\u{1163F}'..='\u{11640}'
            | '\u{116AB}'
            | '\u{116AD}'
            | '\u{116B0}'..='\u{116B5}'
            | '\u{116B7}'
            | '\u{1171D}'..='\u{1171F}'
            | '\u{11722}'..='\u{11725}'
            | '\u{11727}'..='\u{1172B}'
            | '\u{1182F}'..='\u{11837}'
            | '\u{11839}'..='\u{1183A}'
            | '\u{11A01}'..='\u{11A0A}'
            | '\u{11A33}'..='\u{11A38}'
            | '\u{11A3B}'..='\u{11A3E}'
            | '\u{11A47}'
            | '\u{11A51}'..='\u{11A56}'
            | '\u{11A59}'..='\u{11A5B}'
            | '\u{11A8A}'..='\u{11A96}'
            | '\u{11A98}'..='\u{11A99}'
            | '\u{11C30}'..='\u{11C36}'
            | '\u{11C38}'..='\u{11C3D}'
            | '\u{11C3F}'
            | '\u{11C92}'..='\u{11CA7}'
            | '\u{11CAA}'..='\u{11CB0}'
            | '\u{11CB2}'..='\u{11CB3}'
            | '\u{11CB5}'..='\u{11CB6}'
            | '\u{16AF0}'..='\u{16AF4}'
            | '\u{16B30}'..='\u{16B36}'
            | '\u{16F8F}'..='\u{16F92}'
            | '\u{1BC9D}'..='\u{1BC9E}'
            | '\u{1D167}'..='\u{1D169}'
            | '\u{1D17B}'..='\u{1D182}'
            | '\u{1D185}'..='\u{1D18B}'
            | '\u{1D1AA}'..='\u{1D1AD}'
            | '\u{1D242}'..='\u{1D244}'
            | '\u{1DA00}'..='\u{1DA36}'
            | '\u{1DA3B}'..='\u{1DA6C}'
            | '\u{1DA75}'
            | '\u{1DA84}'
            | '\u{1DA9B}'..='\u{1DA9F}'
            | '\u{1DAA1}'..='\u{1DAAF}'
            | '\u{1E000}'..='\u{1E006}'
            | '\u{1E008}'..='\u{1E018}'
            | '\u{1E01B}'..='\u{1E021}'
            | '\u{1E023}'..='\u{1E024}'
            | '\u{1E026}'..='\u{1E02A}'
            | '\u{1E8D0}'..='\u{1E8D6}'
            | '\u{1E944}'..='\u{1E94A}'
            | '\u{E0100}'..='\u{E01EF}'
    )
}

fn is_wide_char(ch: char) -> bool {
    matches!(
        ch,
        '\u{1100}'..='\u{115F}'
            | '\u{231A}'..='\u{231B}'
            | '\u{2329}'..='\u{232A}'
            | '\u{23E9}'..='\u{23EC}'
            | '\u{23F0}'
            | '\u{23F3}'
            | '\u{25FD}'..='\u{25FE}'
            | '\u{2614}'..='\u{2615}'
            | '\u{2648}'..='\u{2653}'
            | '\u{267F}'
            | '\u{2693}'
            | '\u{26A1}'
            | '\u{26AA}'..='\u{26AB}'
            | '\u{26BD}'..='\u{26BE}'
            | '\u{26C4}'..='\u{26C5}'
            | '\u{26CE}'
            | '\u{26D4}'
            | '\u{26EA}'
            | '\u{26F2}'..='\u{26F3}'
            | '\u{26F5}'
            | '\u{26FA}'
            | '\u{26FD}'
            | '\u{2705}'
            | '\u{270A}'..='\u{270B}'
            | '\u{2728}'
            | '\u{274C}'
            | '\u{274E}'
            | '\u{2753}'..='\u{2755}'
            | '\u{2757}'
            | '\u{2795}'..='\u{2797}'
            | '\u{27B0}'
            | '\u{27BF}'
            | '\u{2B1B}'..='\u{2B1C}'
            | '\u{2B50}'
            | '\u{2B55}'
            | '\u{2E80}'..='\u{2E99}'
            | '\u{2E9B}'..='\u{2EF3}'
            | '\u{2F00}'..='\u{2FD5}'
            | '\u{2FF0}'..='\u{2FFB}'
            | '\u{3000}'..='\u{303E}'
            | '\u{3041}'..='\u{3096}'
            | '\u{3099}'..='\u{30FF}'
            | '\u{3105}'..='\u{312F}'
            | '\u{3131}'..='\u{318E}'
            | '\u{3190}'..='\u{31E3}'
            | '\u{31F0}'..='\u{321E}'
            | '\u{3220}'..='\u{3247}'
            | '\u{3250}'..='\u{4DBF}'
            | '\u{4E00}'..='\u{A48C}'
            | '\u{A490}'..='\u{A4C6}'
            | '\u{A960}'..='\u{A97C}'
            | '\u{AC00}'..='\u{D7A3}'
            | '\u{F900}'..='\u{FAFF}'
            | '\u{FE10}'..='\u{FE19}'
            | '\u{FE30}'..='\u{FE52}'
            | '\u{FE54}'..='\u{FE66}'
            | '\u{FE68}'..='\u{FE6B}'
            | '\u{FF01}'..='\u{FF60}'
            | '\u{FFE0}'..='\u{FFE6}'
            | '\u{16FE0}'..='\u{16FE4}'
            | '\u{17000}'..='\u{187F7}'
            | '\u{18800}'..='\u{18CD5}'
            | '\u{18D00}'..='\u{18D08}'
            | '\u{1B000}'..='\u{1B11E}'
            | '\u{1B170}'..='\u{1B2FB}'
            | '\u{1F004}'
            | '\u{1F0CF}'
            | '\u{1F18E}'
            | '\u{1F191}'..='\u{1F19A}'
            | '\u{1F200}'..='\u{1F202}'
            | '\u{1F210}'..='\u{1F23B}'
            | '\u{1F240}'..='\u{1F248}'
            | '\u{1F250}'..='\u{1F251}'
            | '\u{1F260}'..='\u{1F265}'
            | '\u{1F300}'..='\u{1F320}'
            | '\u{1F32D}'..='\u{1F335}'
            | '\u{1F337}'..='\u{1F37C}'
            | '\u{1F37E}'..='\u{1F393}'
            | '\u{1F3A0}'..='\u{1F3CA}'
            | '\u{1F3CF}'..='\u{1F3D3}'
            | '\u{1F3E0}'..='\u{1F3F0}'
            | '\u{1F3F4}'
            | '\u{1F3F8}'..='\u{1F43E}'
            | '\u{1F440}'
            | '\u{1F442}'..='\u{1F4FC}'
            | '\u{1F4FF}'..='\u{1F53D}'
            | '\u{1F54B}'..='\u{1F54E}'
            | '\u{1F550}'..='\u{1F567}'
            | '\u{1F57A}'
            | '\u{1F595}'..='\u{1F596}'
            | '\u{1F5A4}'
            | '\u{1F5FB}'..='\u{1F64F}'
            | '\u{1F680}'..='\u{1F6C5}'
            | '\u{1F6CC}'
            | '\u{1F6D0}'..='\u{1F6D2}'
            | '\u{1F6EB}'..='\u{1F6EC}'
            | '\u{1F6F4}'..='\u{1F6F9}'
            | '\u{1F910}'..='\u{1F93E}'
            | '\u{1F940}'..='\u{1F970}'
            | '\u{1F973}'..='\u{1F976}'
            | '\u{1F97A}'
            | '\u{1F97C}'..='\u{1F9A2}'
            | '\u{1F9B0}'..='\u{1F9B9}'
            | '\u{1F9C0}'..='\u{1F9C2}'
            | '\u{1F9D0}'..='\u{1F9FF}'
            | '\u{20000}'..='\u{2FFFD}'
            | '\u{30000}'..='\u{3FFFD}'
    )
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
    fn cursor_column_uses_display_width_for_chinese_text() {
        let mut input = SlashInput::new(["help"]);
        type_text(&mut input, "你好a");

        let view = input.render();
        assert_eq!(view.cursor_column, 6);
        assert_eq!(visible_text_width(&view.lines[1]), DEFAULT_INPUT_WIDTH + 2);
    }

    #[test]
    fn moving_cursor_over_chinese_text_preserves_wide_character_boundaries() {
        let mut input = SlashInput::new(["help"]);
        type_text(&mut input, "你好a");

        input.handle_key(KeyEvent::plain(KeyCode::Left));
        assert_eq!(input.cursor(), 2);
        assert_eq!(input.render().cursor_column, 5);

        input.handle_key(KeyEvent::plain(KeyCode::Left));
        assert_eq!(input.cursor(), 1);
        assert_eq!(input.render().cursor_column, 3);
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
