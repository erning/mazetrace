use ratatui::layout::{Constraint, Direction as LayoutDirection, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, Phase};
use crate::render::{render_maze_cells, RenderCell, RenderKind};

const STATUS_SEPARATOR: &str = "  ";
const CONTROLS: &str = "Space start/pause  S step  N new  R reset  +/- speed  1-6 solver  Q quit";

pub fn draw(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();
    let block = Block::default().title(" MazeTrace ").borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(LayoutDirection::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(inner);

    if !app.fits() {
        let (required_width, required_height) = app.required_render_size();
        let message = vec![
            Line::from("Terminal is too small for the current maze."),
            Line::from(format!(
                "Need {required_width}x{required_height} maze cells render area; press N after resizing."
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
    let maze_lines = render_maze_cells(
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
        let left_padding = available_width.saturating_sub(line.len()) / 2;
        lines.push(styled_maze_line(&line, left_padding));
    }

    lines
}

fn styled_maze_line(cells: &[RenderCell], left_padding: usize) -> Line<'static> {
    let mut spans = Vec::new();
    if left_padding > 0 {
        spans.push(Span::raw(" ".repeat(left_padding)));
    }

    let mut start = 0;
    while start < cells.len() {
        let kind = cells[start].kind;
        let mut end = start + 1;
        while end < cells.len() && cells[end].kind == kind {
            end += 1;
        }

        let text = cells[start..end]
            .iter()
            .map(|cell| cell.ch)
            .collect::<String>();
        spans.push(Span::styled(text, maze_style(kind)));
        start = end;
    }

    Line::from(spans)
}

fn maze_style(kind: RenderKind) -> Style {
    match kind {
        RenderKind::FinalPath => Style::default()
            .fg(Color::LightYellow)
            .add_modifier(Modifier::BOLD),
        RenderKind::Start | RenderKind::Exit => Style::default()
            .fg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
        RenderKind::ExplorerCurrent => Style::default()
            .fg(Color::LightCyan)
            .add_modifier(Modifier::BOLD),
        RenderKind::GeneratorCurrent => Style::default()
            .fg(Color::LightMagenta)
            .add_modifier(Modifier::BOLD),
        RenderKind::Explored => Style::default().fg(Color::DarkGray),
        RenderKind::Wall => Style::default().fg(Color::Gray),
        RenderKind::Empty => Style::default(),
    }
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
    let maze_width = app.maze().width();
    let maze_height = app.maze().height();

    let status = status_line(
        &[
            StatusField::new("Phase: ", phase),
            StatusField::new("Gen: ", app.generator_algorithm().label()),
            StatusField::new("Solve: ", app.solver_algorithm().label()),
            StatusField::new("Size: ", format!("{maze_width}x{maze_height}")),
            StatusField::new("Steps: ", steps.to_string()),
            StatusField::new("Speed: ", app.speed().to_string()),
            StatusField::activity(activity),
        ],
        usize::from(area.width),
    );

    let controls = Line::from(truncate_text(CONTROLS, usize::from(area.width)));
    let message = Line::from(truncate_text(app.message(), usize::from(area.width)));

    frame.render_widget(Paragraph::new(vec![status, controls, message]), area);
}

struct StatusField {
    label: &'static str,
    value: String,
    value_style: Style,
}

impl StatusField {
    fn new(label: &'static str, value: impl Into<String>) -> Self {
        Self {
            label,
            value: value.into(),
            value_style: Style::default(),
        }
    }

    fn activity(value: impl Into<String>) -> Self {
        Self {
            label: "",
            value: value.into(),
            value_style: Style::default().add_modifier(Modifier::BOLD),
        }
    }

    fn width(&self) -> usize {
        self.label.chars().count() + self.value.chars().count()
    }
}

fn status_line(fields: &[StatusField], max_width: usize) -> Line<'static> {
    if max_width == 0 {
        return Line::from("");
    }

    let mut spans = Vec::new();
    let mut used_width = 0;

    for field in fields {
        let separator_width = if spans.is_empty() {
            0
        } else {
            STATUS_SEPARATOR.len()
        };
        let field_width = field.width();

        if used_width + separator_width + field_width > max_width {
            if spans.is_empty() {
                spans.push(Span::raw(truncate_text(
                    format!("{}{}", field.label, field.value),
                    max_width,
                )));
            }
            break;
        }

        if !spans.is_empty() {
            spans.push(Span::raw(STATUS_SEPARATOR));
            used_width += separator_width;
        }

        if !field.label.is_empty() {
            spans.push(Span::styled(
                field.label,
                Style::default().fg(Color::DarkGray),
            ));
        }
        spans.push(Span::styled(field.value.clone(), field.value_style));
        used_width += field_width;
    }

    Line::from(spans)
}

fn truncate_text(text: impl AsRef<str>, max_width: usize) -> String {
    text.as_ref().chars().take(max_width).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_line_omits_fields_that_do_not_fit() {
        let line = status_line(
            &[
                StatusField::new("Phase: ", "Generating"),
                StatusField::new("Generator: ", "Recursive Division"),
            ],
            18,
        );

        assert_eq!(line_width(&line), 17);
        assert_eq!(line_text(&line), "Phase: Generating");
    }

    #[test]
    fn status_line_truncates_first_field_when_terminal_is_tiny() {
        let line = status_line(&[StatusField::new("Phase: ", "Generating")], 8);

        assert_eq!(line_width(&line), 8);
        assert_eq!(line_text(&line), "Phase: G");
    }

    fn line_width(line: &Line<'_>) -> usize {
        line.spans
            .iter()
            .map(|span| span.content.chars().count())
            .sum()
    }

    fn line_text(line: &Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }
}
