use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Paragraph},
    Terminal,
};
use std::{io, panic};

fn main() -> io::Result<()> {
    // 1. 注册 Panic 钩子
    // 如果程序崩溃，它会自动恢复终端状态，否则你的终端会变乱码
    let panic_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, crossterm::cursor::Show);
        panic_hook(panic_info);
    }));

    // 2. 初始化终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, crossterm::cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 3. 程序状态
    let mut input_text = String::from("Find and fix a bug in @filename");

    // 4. 主循环
    loop {
        terminal.draw(|f| {
            // 垂直布局：Header(6) -> Tip(2) -> Input(3)
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(6),
                    Constraint::Length(2),
                    Constraint::Length(3),
                ])
                .split(f.size());

            // --- 绘制 Header ---
            let header_text = vec![
                Line::from(vec![
                    Span::raw(">_ "),
                    Span::styled("Local Coder ", Style::default().bold()),
                    Span::styled("(v0.1.0)", Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(""), // 空行
                Line::from(vec![
                    Span::styled("model:     ", Style::default().fg(Color::DarkGray)),
                    Span::raw("GLM-5.4 medium    "),
                    Span::styled("/model ", Style::default().fg(Color::Cyan)),
                    Span::styled("to change", Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(vec![
                    Span::styled("directory: ", Style::default().fg(Color::DarkGray)),
                    Span::raw("~/github/project"),
                ]),
            ];

            let header_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray));

            f.render_widget(Paragraph::new(header_text).block(header_block), chunks[0]);

            // --- 绘制 Tip ---
            let tip_line = Line::from(vec![
                Span::styled("Tip: ", Style::default().bold().fg(Color::White)),
                Span::styled("New ", Style::default().add_modifier(Modifier::ITALIC).fg(Color::White)),
                Span::styled(
                    "For a limited time, Codex is included in your plan for free – let's build together.",
                    Style::default().fg(Color::DarkGray),
                ),
            ]);
            f.render_widget(Paragraph::new(tip_line), chunks[1]);

            // --- 绘制 Input (Prompt) ---
            // 模拟图中底部带光标的样式
            let input_display = Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::White)),
                Span::styled(&input_text, Style::default().bold().fg(Color::White)),
            ]);
            f.render_widget(Paragraph::new(input_display), chunks[2]);

            // 5. 设置光标
            // 这里的 +2 是因为前面的 "> " 占了两个字符
            f.set_cursor(
                chunks[2].x + input_text.len() as u16 + 2,
                chunks[2].y,
            );
        })?;

        // 6. 事件监听
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                // 只处理按下事件，忽略释放事件（Windows 兼容性）
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        // 退出逻辑
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                        KeyCode::Esc => break,
                        
                        // 输入逻辑
                        KeyCode::Char(c) => {
                            input_text.push(c);
                        }
                        KeyCode::Backspace => {
                            input_text.pop();
                        }
                        KeyCode::Enter => {
                            // 这里处理提交逻辑，目前先清空作为演示
                            input_text.clear();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // --- 恢复终端状态 ---
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::cursor::Show
    )?;

    Ok(())
}