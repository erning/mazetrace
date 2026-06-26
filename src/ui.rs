use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, Phase};
use crate::render::render_maze;

pub fn draw(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();
    let block = Block::default().title(" MazeTrace ").borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(inner);

    if !app.fits() {
        let (required_width, required_height) = app.required_render_size();
        let message = vec![
            Line::from("Terminal is too small for the current maze."),
            Line::from(format!(
                "Need {}x{} maze cells render area; press N after resizing.",
                required_width, required_height
            )),
            Line::from(app.message().to_string()),
        ];
        frame.render_widget(
            Paragraph::new(message).style(Style::default().fg(Color::Yellow)),
            chunks[0],
        );
        draw_status(frame, app, chunks[1]);
        return;
    }

    let lines = centered_maze_lines(app, chunks[0].width as usize, chunks[0].height as usize);
    frame.render_widget(Paragraph::new(lines), chunks[0]);
    draw_status(frame, app, chunks[1]);
}

fn centered_maze_lines(
    app: &App,
    available_width: usize,
    available_height: usize,
) -> Vec<Line<'static>> {
    let maze_lines = render_maze(
        app.maze(),
        app.generator(),
        app.explorer(),
        app.render_phase(),
        app.ascii(),
    );
    let top_padding = available_height.saturating_sub(maze_lines.len()) / 2;
    let mut lines = Vec::with_capacity(available_height.max(maze_lines.len()));

    for _ in 0..top_padding {
        lines.push(Line::from(""));
    }

    for line in maze_lines {
        let left_padding = available_width.saturating_sub(line.chars().count()) / 2;
        lines.push(Line::from(format!("{}{}", " ".repeat(left_padding), line)));
    }

    lines
}

fn draw_status(frame: &mut Frame<'_>, app: &App, area: ratatui::layout::Rect) {
    let phase = match app.phase() {
        Phase::Generating => "Generating",
        Phase::Ready => "Ready",
        Phase::Exploring => "Exploring",
        Phase::Solved => "Solved",
        Phase::Failed => "Failed",
    };
    let activity = match app.phase() {
        Phase::Ready => "Waiting",
        Phase::Solved => "Done",
        Phase::Failed => "Stopped",
        _ if app.paused() => "Paused",
        _ => "Running",
    };
    let steps = match app.phase() {
        Phase::Generating | Phase::Ready => app.generator().step_count(),
        Phase::Exploring | Phase::Solved | Phase::Failed => app.explorer().step_count(),
    };

    let status = Line::from(vec![
        Span::styled("Phase: ", Style::default().fg(Color::DarkGray)),
        Span::raw(phase),
        Span::raw("    "),
        Span::styled("Gen: ", Style::default().fg(Color::DarkGray)),
        Span::raw(app.generator_algorithm().label()),
        Span::raw("    "),
        Span::styled("Solve: ", Style::default().fg(Color::DarkGray)),
        Span::raw(app.solver_algorithm().label()),
        Span::raw("    "),
        Span::styled("Size: ", Style::default().fg(Color::DarkGray)),
        Span::raw(format!("{}x{}", app.maze().width(), app.maze().height())),
        Span::raw("    "),
        Span::styled("Steps: ", Style::default().fg(Color::DarkGray)),
        Span::raw(steps.to_string()),
        Span::raw("    "),
        Span::styled("Speed: ", Style::default().fg(Color::DarkGray)),
        Span::raw(app.speed().to_string()),
        Span::raw("    "),
        Span::styled(activity, Style::default().add_modifier(Modifier::BOLD)),
    ]);

    let controls =
        Line::from("Space start/pause  S step  N new  R reset  +/- speed  1-4 solver  Q quit");
    let message = Line::from(app.message().to_string());

    frame.render_widget(Paragraph::new(vec![status, controls, message]), area);
}
