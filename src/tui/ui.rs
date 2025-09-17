use crate::core::breathing::BreathPhase;
use crate::integrations::DndState;
use crate::tui::app::{App, AppMode};
use ratatui::{
    prelude::*,
    symbols,
    widgets::{
        canvas::{Canvas, Circle, Context},
        Block, Borders, Gauge, Paragraph,
    },
};

pub fn draw(frame: &mut Frame, app: &App) {
    // Check if we have a status message to display
    let has_status = app.status_message().is_some();

    let chunks = if has_status {
        // Check if we have setup instructions that need more space
        let status_height = if let Some(msg) = app.status_message() {
            if msg.contains("📋 Focus Mode Setup Instructions") {
                20 // More space for setup instructions (increased from 12)
            } else {
                4 // Normal space for other status messages
            }
        } else {
            4
        };

        Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(5),
                Constraint::Length(status_height), // Dynamic status message area
            ])
            .split(frame.size())
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(5),
            ])
            .split(frame.size())
    };

    // Title with Focus mode status and audio status
    let focus_indicator = if app.is_dnd_supported() {
        match app.dnd_state() {
            DndState::Enabled => " 🔕",
            DndState::Disabled => " 🔔",
            DndState::Unknown => " ❓",
        }
    } else {
        ""
    };

    let audio_indicator = if cfg!(feature = "audio") {
        if app.audio_is_muted() {
            " 🔇"
        } else if !app.audio_is_available() {
            " 🚫"
        } else {
            " 🔊"
        }
    } else {
        ""
    };

    let title = match app.mode() {
        AppMode::Pomodoro => format!(
            "🍅 Pomodoro Timer - Session #{}{}{}",
            app.session_count() + 1,
            focus_indicator,
            audio_indicator
        ),
        AppMode::Break => {
            let break_type = if (app.session_count() % 4) == 0 {
                "Long Break"
            } else {
                "Short Break"
            };
            format!(
                "☕ {} - After Session #{}{}{}",
                break_type,
                app.session_count(),
                focus_indicator,
                audio_indicator
            )
        }
    };

    let title_widget = Paragraph::new(title)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(title_widget, chunks[0]);

    // Main content
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(3),
        ])
        .split(chunks[1]);

    // Timer display
    render_timer(frame, app, main_chunks[0]);

    // Progress bar
    render_progress(frame, app, main_chunks[1]);

    // Breathing or status
    if app.mode() == AppMode::Break {
        render_breathing(frame, app, main_chunks[2]);
    } else {
        render_status(frame, app, main_chunks[2]);
    }

    // Controls
    render_controls(frame, app, chunks[2]);

    // Status message (if any)
    if has_status {
        if let Some(msg) = app.status_message() {
            let status_style = if msg.contains("❌") || msg.contains("⚠️") {
                Style::default().fg(Color::Red).bold()
            } else if msg.contains("✅") {
                Style::default().fg(Color::Green).bold()
            } else if msg.contains("❓") {
                Style::default().fg(Color::Cyan).bold()
            } else {
                Style::default().fg(Color::Yellow).bold()
            };

            // Split message into lines and limit length for better formatting
            let lines: Vec<&str> = msg.lines().collect();
            let is_setup_instructions = msg.contains("📋 Focus Mode Setup Instructions");
            let max_lines = if is_setup_instructions { 30 } else { 3 };

            let display_text = if lines.len() > max_lines {
                // If too many lines, show first few and indicate more
                let mut displayed_lines = Vec::new();
                for i in 0..max_lines {
                    if let Some(line) = lines.get(i) {
                        displayed_lines.push(*line);
                    }
                }
                format!(
                    "{}\n... (Press 'ESC' to dismiss)",
                    displayed_lines.join("\n")
                )
            } else {
                format!("{}\n(Press 'ESC' to dismiss)", msg)
            };

            // Determine wrapping behavior based on content type
            let wrap_config = if msg.contains("📋 Focus Mode Setup Instructions") {
                // Don't trim whitespace for setup instructions to preserve indentation
                ratatui::widgets::Wrap { trim: false }
            } else {
                // Use normal trimming for other messages
                ratatui::widgets::Wrap { trim: true }
            };

            let status_widget = Paragraph::new(display_text)
                .style(status_style)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .title(" Status - Press ESC to dismiss ")
                        .title_alignment(Alignment::Center),
                )
                .alignment(Alignment::Left) // Left align for better readability
                .wrap(wrap_config);

            frame.render_widget(status_widget, chunks[3]);
        }
    }
}

fn render_timer(frame: &mut Frame, app: &App, area: Rect) {
    let timer = app.timer();
    let remaining = timer.remaining();
    let minutes = remaining.as_secs() / 60;
    let seconds = remaining.as_secs() % 60;

    let time_str = format!("{:02}:{:02}", minutes, seconds);
    let color = match timer.state() {
        crate::core::timer::TimerState::Running => Color::Green,
        crate::core::timer::TimerState::Paused => Color::Yellow,
        crate::core::timer::TimerState::Completed => Color::Red,
        crate::core::timer::TimerState::Idle => Color::Gray,
    };

    let timer_widget = Paragraph::new(time_str)
        .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Time Remaining"),
        );

    frame.render_widget(timer_widget, area);
}

fn render_progress(frame: &mut Frame, app: &App, area: Rect) {
    let progress = (app.timer().progress() * 100.0) as u16;

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Progress"))
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent(progress);

    frame.render_widget(gauge, area);
}

fn render_breathing(frame: &mut Frame, app: &App, area: Rect) {
    // Only show breathing exercise when timer is running
    if app.timer().state() == crate::core::timer::TimerState::Running {
        if let Some(exercise) = app.breathing_exercise() {
            // Split the area for circle and text
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(10),   // Circle area
                    Constraint::Length(4), // Info area
                ])
                .split(area);

            // Render the breathing circle
            render_breathing_circle(frame, exercise, chunks[0]);

            // Render breathing info
            let instruction = exercise.get_instruction();
            let pattern = exercise.get_pattern_name();
            let cycles = exercise.get_cycle_count();
            let remaining = exercise.get_remaining_in_phase();

            // Calculate breathing session progress if applicable
            let session_info = if let Some(duration) = app.breathing_duration() {
                let elapsed = exercise.get_total_elapsed();
                let remaining_session = duration.saturating_sub(elapsed);
                format!(" | Session: {:.0}s left", remaining_session.as_secs_f64())
            } else {
                String::new()
            };

            let content = vec![
                Line::from(vec![Span::styled(
                    instruction,
                    Style::default()
                        .fg(get_phase_color(exercise.get_current_phase()))
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(format!(
                    "{} | Cycle {}{}",
                    pattern,
                    cycles + 1,
                    session_info
                )),
                Line::from(format!("{:.0}s", remaining.as_secs_f64())),
            ];

            let info_widget = Paragraph::new(content)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::TOP));

            frame.render_widget(info_widget, chunks[1]);
        } else if app.breathing_complete() {
            // Show rest mode
            let break_widget = Paragraph::new(vec![
                Line::from("☕ Rest and Stretch"),
                Line::from(""),
                Line::from("Take this time to:"),
                Line::from("• Stand up and stretch"),
                Line::from("• Look away from the screen"),
                Line::from("• Hydrate"),
                Line::from(""),
                Line::from("Press 'T' to toggle breathing exercises"),
            ])
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Break Time"));

            frame.render_widget(break_widget, area);
        } else {
            let break_widget = Paragraph::new("Enjoy your break!")
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("Break Time"));

            frame.render_widget(break_widget, area);
        }
    } else if app.timer().state() == crate::core::timer::TimerState::Idle {
        // Show instructions when break timer is idle
        let selected_pattern = app
            .breathing_exercise()
            .map(|ex| ex.get_pattern())
            .unwrap_or(crate::core::BreathingPattern::Simple);

        // Create pattern options with visual indicators for selection
        let simple_line = if selected_pattern == crate::core::BreathingPattern::Simple {
            Line::from(vec![
                Span::styled(
                    "✓ ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("1: "),
                Span::styled(
                    "Simple (4-4)",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ])
        } else {
            Line::from(vec![
                Span::raw("  1: "),
                Span::styled("Simple (4-4)", Style::default().fg(Color::Cyan)),
            ])
        };

        let coherent_line = if selected_pattern == crate::core::BreathingPattern::Coherent {
            Line::from(vec![
                Span::styled(
                    "✓ ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("2: "),
                Span::styled(
                    "Coherent (5-5)",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ])
        } else {
            Line::from(vec![
                Span::raw("  2: "),
                Span::styled("Coherent (5-5)", Style::default().fg(Color::Cyan)),
            ])
        };

        let short_box_line = if selected_pattern == crate::core::BreathingPattern::ShortBox {
            Line::from(vec![
                Span::styled(
                    "✓ ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("3: "),
                Span::styled(
                    "Short Box (3-3-3-3)",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ])
        } else {
            Line::from(vec![
                Span::raw("  3: "),
                Span::styled("Short Box (3-3-3-3)", Style::default().fg(Color::Cyan)),
            ])
        };

        let content = vec![
            Line::from(""),
            Line::from("Choose a breathing pattern:"),
            Line::from(""),
            simple_line,
            coherent_line,
            short_box_line,
            Line::from(""),
            Line::from(vec![Span::styled(
                "Press Space to start break",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )]),
        ];

        let break_widget = Paragraph::new(content)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Break Ready"));

        frame.render_widget(break_widget, area);
    } else {
        // Paused or completed state
        let break_widget = Paragraph::new("Break time")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Break"));

        frame.render_widget(break_widget, area);
    }
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let state = match (app.mode(), app.timer().state()) {
        (crate::tui::app::AppMode::Pomodoro, crate::core::timer::TimerState::Idle) => {
            "Ready to start Pomodoro - Press Space"
        }
        (crate::tui::app::AppMode::Pomodoro, crate::core::timer::TimerState::Running) => {
            "Focus time - stay concentrated!"
        }
        (crate::tui::app::AppMode::Pomodoro, crate::core::timer::TimerState::Paused) => {
            "Paused - Press Space to resume"
        }
        (crate::tui::app::AppMode::Pomodoro, crate::core::timer::TimerState::Completed) => {
            "Pomodoro complete! Press Space to prepare break"
        }
        (crate::tui::app::AppMode::Break, crate::core::timer::TimerState::Idle) => {
            "Break ready - Choose pattern (1-3) and press Space"
        }
        (crate::tui::app::AppMode::Break, crate::core::timer::TimerState::Running) => {
            "Break in progress - Relax and breathe"
        }
        (crate::tui::app::AppMode::Break, crate::core::timer::TimerState::Paused) => {
            "Break paused - Press Space to resume"
        }
        (crate::tui::app::AppMode::Break, crate::core::timer::TimerState::Completed) => {
            "Break complete! Press Space to start new Pomodoro"
        }
    };

    let color = match app.timer().state() {
        crate::core::timer::TimerState::Idle => Color::Yellow,
        crate::core::timer::TimerState::Running => Color::Green,
        crate::core::timer::TimerState::Paused => Color::Yellow,
        crate::core::timer::TimerState::Completed => Color::Cyan,
    };

    let status_widget = Paragraph::new(state)
        .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Status"));

    frame.render_widget(status_widget, area);
}

fn render_controls(frame: &mut Frame, app: &App, area: Rect) {
    let terminal_width = area.width;
    let controls = get_responsive_controls(app, terminal_width);

    let controls_widget = Paragraph::new(controls)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Controls"));

    frame.render_widget(controls_widget, area);
}

fn get_responsive_controls(app: &App, width: u16) -> Vec<Line<'static>> {
    match width {
        0..=39 => get_narrow_controls(app),
        40..=79 => get_medium_controls(app),
        _ => get_wide_controls(app),
    }
}

fn get_wide_controls(app: &App) -> Vec<Line<'static>> {
    if app.mode() == AppMode::Break {
        let is_long_break =
            (app.session_count() % 4) == 0 && app.timer().duration().as_secs() > 5 * 60;
        let is_shortened_break = app.break_was_shortened();

        let mut first_line = vec![
            Span::raw("Space: "),
            Span::styled("Start/Pause", Style::default().fg(Color::Green)),
            Span::raw(" | "),
            Span::raw("R: "),
            Span::styled("Reset", Style::default().fg(Color::Yellow)),
            Span::raw(" | "),
            Span::raw("B: "),
            Span::styled("Skip Break", Style::default().fg(Color::Cyan)),
        ];

        if is_long_break {
            first_line.extend(vec![
                Span::raw(" | "),
                Span::raw("H: "),
                Span::styled("Shorten", Style::default().fg(Color::Magenta)),
            ]);
        }

        if is_shortened_break {
            first_line.extend(vec![
                Span::raw(" | "),
                Span::raw("E: "),
                Span::styled("Extend", Style::default().fg(Color::Blue)),
            ]);
        }

        first_line.extend(vec![
            Span::raw(" | "),
            Span::raw("Q/Esc: "),
            Span::styled("Quit", Style::default().fg(Color::Red)),
        ]);

        let mut second_line = vec![];

        // Show breathing controls if in break mode
        if app.breathing_enabled() && app.mode() == AppMode::Break {
            second_line.extend(vec![
                Span::raw("X: "),
                Span::styled("Skip Breathing", Style::default().fg(Color::Magenta)),
                Span::raw(" | "),
            ]);
        }

        second_line.extend(vec![
            Span::raw("T: "),
            if app.breathing_enabled() {
                Span::styled("Breathing ON", Style::default().fg(Color::Green))
            } else {
                Span::styled("Breathing OFF", Style::default().fg(Color::DarkGray))
            },
        ]);

        // Add audio controls if feature is enabled
        if cfg!(feature = "audio") {
            second_line.extend(vec![
                Span::raw(" | "),
                Span::raw("M: "),
                Span::styled("Mute", Style::default().fg(Color::Magenta)),
                Span::raw(" | "),
                Span::raw("+/-: "),
                Span::styled("Volume", Style::default().fg(Color::Green)),
                Span::raw(" | "),
                Span::raw("V: "),
                Span::styled("Test", Style::default().fg(Color::Yellow)),
            ]);
        }

        second_line.extend(vec![
            Span::raw(" | "),
            Span::raw("C: "),
            Span::styled("Clear", Style::default().fg(Color::Gray)),
            Span::raw(" | "),
            Span::raw("F: "),
            Span::styled("Focus Help", Style::default().fg(Color::Cyan)),
        ]);

        // Add Focus controls if supported
        if app.is_dnd_supported() {
            second_line.extend(vec![
                Span::raw(" | "),
                Span::raw("D: "),
                Span::styled("Toggle Focus", Style::default().fg(Color::LightBlue)),
                Span::raw(" | "),
                Span::raw("A: "),
                Span::styled(
                    if app.dnd_auto_enabled() {
                        "Auto ON"
                    } else {
                        "Auto OFF"
                    },
                    Style::default().fg(if app.dnd_auto_enabled() {
                        Color::Green
                    } else {
                        Color::Gray
                    }),
                ),
            ]);
        }

        vec![Line::from(first_line), Line::from(second_line)]
    } else {
        let mut first_line = vec![
            Span::raw("Space: "),
            Span::styled("Start/Pause", Style::default().fg(Color::Green)),
            Span::raw(" | "),
            Span::raw("R: "),
            Span::styled("Reset", Style::default().fg(Color::Yellow)),
            Span::raw(" | "),
            Span::raw("S: "),
            Span::styled("Skip to Break", Style::default().fg(Color::Cyan)),
        ];

        // Add Focus controls if supported
        if app.is_dnd_supported() {
            first_line.extend(vec![
                Span::raw(" | "),
                Span::raw("D: "),
                Span::styled("Toggle Focus", Style::default().fg(Color::LightBlue)),
                Span::raw(" | "),
                Span::raw("A: "),
                Span::styled(
                    if app.dnd_auto_enabled() {
                        "Auto ON"
                    } else {
                        "Auto OFF"
                    },
                    Style::default().fg(if app.dnd_auto_enabled() {
                        Color::Green
                    } else {
                        Color::Gray
                    }),
                ),
            ]);
        }

        first_line.extend(vec![
            Span::raw(" | "),
            Span::raw("Q/Esc: "),
            Span::styled("Quit", Style::default().fg(Color::Red)),
        ]);

        if cfg!(feature = "audio") {
            let audio_line = vec![
                Span::raw("M: "),
                Span::styled("Mute", Style::default().fg(Color::Magenta)),
                Span::raw(" | "),
                Span::raw("+/-: "),
                Span::styled("Volume", Style::default().fg(Color::Green)),
                Span::raw(" | "),
                Span::raw("T: "),
                Span::styled("Test Audio", Style::default().fg(Color::Yellow)),
                Span::raw(" | "),
                Span::raw("C: "),
                Span::styled("Clear", Style::default().fg(Color::Gray)),
                Span::raw(" | "),
                Span::raw("F: "),
                Span::styled("Focus Help", Style::default().fg(Color::Cyan)),
            ];

            vec![Line::from(first_line), Line::from(audio_line)]
        } else {
            vec![Line::from(first_line)]
        }
    }
}

fn get_medium_controls(app: &App) -> Vec<Line<'static>> {
    if app.mode() == AppMode::Break {
        let is_long_break =
            (app.session_count() % 4) == 0 && app.timer().duration().as_secs() > 5 * 60;
        let is_shortened_break = app.break_was_shortened();

        let mut first_line = vec![
            Span::raw("Space: "),
            Span::styled("Start", Style::default().fg(Color::Green)),
            Span::raw(" | "),
            Span::raw("R: "),
            Span::styled("Reset", Style::default().fg(Color::Yellow)),
            Span::raw(" | "),
            Span::raw("B: "),
            Span::styled("Skip", Style::default().fg(Color::Cyan)),
        ];

        if is_long_break {
            first_line.extend(vec![
                Span::raw(" | "),
                Span::raw("H: "),
                Span::styled("Shorten", Style::default().fg(Color::Magenta)),
            ]);
        }

        if is_shortened_break {
            first_line.extend(vec![
                Span::raw(" | "),
                Span::raw("E: "),
                Span::styled("Extend", Style::default().fg(Color::Blue)),
            ]);
        }

        first_line.extend(vec![
            Span::raw(" | "),
            Span::raw("Q: "),
            Span::styled("Quit", Style::default().fg(Color::Red)),
        ]);

        let mut second_line = vec![
            Span::raw("1: "),
            Span::styled("Simple", Style::default().fg(Color::Cyan)),
            Span::raw(" | "),
            Span::raw("2: "),
            Span::styled("Box", Style::default().fg(Color::Cyan)),
            Span::raw(" | "),
            Span::raw("3: "),
            Span::styled("4-7-8", Style::default().fg(Color::Cyan)),
        ];

        // Add audio controls if feature is enabled
        if cfg!(feature = "audio") {
            second_line.extend(vec![
                Span::raw(" | "),
                Span::raw("M: "),
                Span::styled("Mute", Style::default().fg(Color::Magenta)),
            ]);
        }

        // Add Focus control for medium displays (condensed)
        if app.is_dnd_supported() {
            second_line.extend(vec![
                Span::raw(" | "),
                Span::raw("D: "),
                Span::styled("Focus", Style::default().fg(Color::LightBlue)),
            ]);
        }

        vec![Line::from(first_line), Line::from(second_line)]
    } else {
        let first_line = vec![
            Span::raw("Space: "),
            Span::styled("Start", Style::default().fg(Color::Green)),
            Span::raw(" | "),
            Span::raw("R: "),
            Span::styled("Reset", Style::default().fg(Color::Yellow)),
            Span::raw(" | "),
            Span::raw("S: "),
            Span::styled("Skip", Style::default().fg(Color::Cyan)),
        ];

        let mut second_line = vec![];

        // Add audio controls if feature is enabled
        if cfg!(feature = "audio") {
            second_line.extend(vec![
                Span::raw("M: "),
                Span::styled("Mute", Style::default().fg(Color::Magenta)),
                Span::raw(" | "),
                Span::raw("+/-: "),
                Span::styled("Vol", Style::default().fg(Color::Green)),
                Span::raw(" | "),
            ]);
        }

        second_line.extend(vec![
            Span::raw("Q/Esc: "),
            Span::styled("Quit", Style::default().fg(Color::Red)),
        ]);

        // Add Focus control to second line for Pomodoro mode
        if app.is_dnd_supported() {
            second_line.extend(vec![
                Span::raw(" | "),
                Span::raw("D: "),
                Span::styled("Focus", Style::default().fg(Color::LightBlue)),
            ]);
        }

        vec![Line::from(first_line), Line::from(second_line)]
    }
}

fn get_narrow_controls(app: &App) -> Vec<Line<'static>> {
    if app.mode() == AppMode::Break {
        let is_long_break =
            (app.session_count() % 4) == 0 && app.timer().duration().as_secs() > 5 * 60;
        let is_shortened_break = app.break_was_shortened();

        let mut second_line = vec![
            Span::raw("B: "),
            Span::styled("Skip", Style::default().fg(Color::Cyan)),
        ];

        if is_long_break {
            second_line.extend(vec![
                Span::raw(" | "),
                Span::raw("H: "),
                Span::styled("Shorten", Style::default().fg(Color::Magenta)),
            ]);
        }

        if is_shortened_break {
            second_line.extend(vec![
                Span::raw(" | "),
                Span::raw("E: "),
                Span::styled("Extend", Style::default().fg(Color::Blue)),
            ]);
        }

        // Add Focus to second line if there's space
        if app.is_dnd_supported() {
            second_line.extend(vec![
                Span::raw(" | "),
                Span::raw("D: "),
                Span::styled("Focus", Style::default().fg(Color::LightBlue)),
            ]);
        }

        second_line.extend(vec![
            Span::raw(" | "),
            Span::raw("Q: "),
            Span::styled("Quit", Style::default().fg(Color::Red)),
        ]);

        vec![
            Line::from(vec![
                Span::raw("␣: "),
                Span::styled("Start", Style::default().fg(Color::Green)),
                Span::raw(" | "),
                Span::raw("R: "),
                Span::styled("Reset", Style::default().fg(Color::Yellow)),
            ]),
            Line::from(second_line),
            Line::from(vec![
                Span::raw("1: "),
                Span::styled("Simple", Style::default().fg(Color::Cyan)),
                Span::raw(" | "),
                Span::raw("2: "),
                Span::styled("Box", Style::default().fg(Color::Cyan)),
                Span::raw(" | "),
                Span::raw("3: "),
                Span::styled("4-7-8", Style::default().fg(Color::Cyan)),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::raw("␣: "),
                Span::styled("Start", Style::default().fg(Color::Green)),
                Span::raw(" | "),
                Span::raw("R: "),
                Span::styled("Reset", Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::raw("S: "),
                Span::styled("Skip", Style::default().fg(Color::Cyan)),
                Span::raw(" | "),
                Span::raw("Q: "),
                Span::styled("Quit", Style::default().fg(Color::Red)),
            ]),
        ]
    }
}

fn render_breathing_circle(
    frame: &mut Frame,
    exercise: &crate::core::BreathingExercise,
    area: Rect,
) {
    let progress = exercise.get_phase_progress();
    let phase = exercise.get_current_phase();

    // Calculate circle radius based on breath phase
    let base_radius = 4.0;
    let max_expansion = 8.0; // Reduced for better visibility

    // Apply easing for smoother animation
    let eased_progress = ease_in_out(progress);

    let radius = match phase {
        BreathPhase::Inhale => {
            // Expand from base to max with easing
            base_radius + (eased_progress * max_expansion)
        }
        BreathPhase::Hold => {
            // Stay at max size
            base_radius + max_expansion
        }
        BreathPhase::Exhale => {
            // Contract from max to base with easing
            base_radius + max_expansion * (1.0 - eased_progress)
        }
        BreathPhase::Rest => {
            // Stay at base size
            base_radius
        }
        BreathPhase::Transition => {
            // Hold at appropriate size - don't make it larger than breathing phases
            if exercise.is_post_exhale_transition() {
                // After exhale, hold at contracted size
                base_radius
            } else {
                // After inhale, hold at expanded size
                base_radius + max_expansion
            }
        }
    };

    let color = get_phase_color(phase);

    let canvas = Canvas::default()
        .block(Block::default().borders(Borders::NONE))
        .x_bounds([-15.0, 15.0])
        .y_bounds([-15.0, 15.0])
        .marker(symbols::Marker::Braille)
        .paint(move |ctx: &mut Context| {
            // Draw the breathing circle
            ctx.draw(&Circle {
                x: 0.0,
                y: 0.0,
                radius,
                color,
            });

            // Draw a smaller inner circle for visual appeal
            if radius > 3.0 {
                ctx.draw(&Circle {
                    x: 0.0,
                    y: 0.0,
                    radius: radius * 0.3,
                    color: Color::DarkGray,
                });
            }
        });

    frame.render_widget(canvas, area);
}

fn get_phase_color(phase: BreathPhase) -> Color {
    match phase {
        BreathPhase::Inhale => Color::Green,
        BreathPhase::Hold => Color::Yellow,
        BreathPhase::Exhale => Color::Blue,
        BreathPhase::Rest => Color::Magenta,
        BreathPhase::Transition => Color::Cyan, // Soft transition color
    }
}

// Easing function for smoother animations
fn ease_in_out(t: f64) -> f64 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - 2.0 * (1.0 - t) * (1.0 - t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::app::App;

    #[test]
    fn test_responsive_controls_wide_terminal() {
        let app = App::new().unwrap();
        let controls = get_responsive_controls(&app, 100);

        // Should contain all expected controls in first line
        let control_text = format!("{:?}", controls[0]);
        assert!(control_text.contains("Start/Pause"));
        assert!(control_text.contains("Skip to Break"));
        assert!(control_text.contains("Reset"));
        assert!(control_text.contains("Quit"));

        if cfg!(feature = "audio") {
            // Wide terminal should have two lines for Pomodoro mode (main controls + audio controls)
            assert_eq!(controls.len(), 2);

            // Should contain audio controls in second line
            let audio_text = format!("{:?}", controls[1]);
            assert!(audio_text.contains("Mute"));
            assert!(audio_text.contains("Volume"));
            assert!(audio_text.contains("Test Audio"));
        } else {
            // Without audio feature, should have single line
            assert_eq!(controls.len(), 1);
        }
    }

    #[test]
    fn test_responsive_controls_wide_terminal_break_mode() {
        let mut app = App::new().unwrap();
        app.skip_to_break(); // Switch to break mode

        let controls = get_responsive_controls(&app, 100);

        // Wide terminal break mode should have 2 lines
        assert_eq!(controls.len(), 2);

        // Should contain skip break control
        let control_text = format!("{:?}", controls[0]);
        assert!(control_text.contains("Skip Break"));
    }

    #[test]
    fn test_responsive_controls_medium_terminal() {
        let app = App::new().unwrap();
        let controls = get_responsive_controls(&app, 50);

        // Medium terminal should have 2 lines for Pomodoro mode
        assert_eq!(controls.len(), 2);

        // Should contain shortened text
        let control_text = format!("{:?}", controls[0]);
        assert!(control_text.contains("Start")); // Shortened from "Start/Pause"
        assert!(!control_text.contains("Start/Pause"));
    }

    #[test]
    fn test_responsive_controls_narrow_terminal() {
        let mut app = App::new().unwrap();
        app.skip_to_break(); // Switch to break mode

        let controls = get_responsive_controls(&app, 30);

        // Narrow terminal break mode should have 3 lines
        assert_eq!(controls.len(), 3);

        // Should use space symbol instead of "Space"
        let control_text = format!("{:?}", controls[0]);
        assert!(control_text.contains("␣"));
        assert!(!control_text.contains("Space"));
    }

    #[test]
    fn test_responsive_controls_edge_cases() {
        let app = App::new().unwrap();

        // Test that we're actually using different control formats
        let narrow_controls = get_responsive_controls(&app, 39);
        let wide_controls = get_responsive_controls(&app, 80);

        // Controls should have different content even if line count is same
        let narrow_text = format!("{:?}", narrow_controls[0]);
        let wide_text = format!("{:?}", wide_controls[0]);

        // Narrow should use space symbol, wide should use "Space:"
        assert!(narrow_text.contains("␣"));
        assert!(!wide_text.contains("␣"));
        assert!(wide_text.contains("Space"));

        // Test with break mode for clearer differences
        let mut break_app = App::new().unwrap();
        break_app.skip_to_break();

        let narrow_break = get_responsive_controls(&break_app, 39);
        let wide_break = get_responsive_controls(&break_app, 80);

        // Break mode should have different number of lines
        assert_eq!(narrow_break.len(), 3); // Most lines for narrow
        assert_eq!(wide_break.len(), 2); // Fewer lines for wide
    }
}
