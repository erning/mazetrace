use ratatui::layout::{Constraint, Direction as LayoutDirection, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, Phase};
use crate::render::{render_maze_cells, RenderCell, RenderKind};

const STATUS_SEPARATOR: &str = "  ";
const CONTROLS: &str = "Space start/pause  S step  N new  R reset  +/- speed  1-6 solver  Q quit";

// Field priorities drive which status-bar fields survive a narrow terminal.
// Higher numbers are kept longer. The solver name and step count carry the
// data users compare, so they outlast phase/size; speed and generator are the
// first to be dropped.
const DEFAULT_PRIORITY: u8 = 50;
const PRIORITY_SOLVE: u8 = 100;
const PRIORITY_STEPS: u8 = 95;
const PRIORITY_PHASE: u8 = 90;
const PRIORITY_SIZE: u8 = 70;
const PRIORITY_ACTIVITY: u8 = 60;
const PRIORITY_GEN: u8 = 45;
const PRIORITY_SPEED: u8 = 30;

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
            StatusField::new("Phase: ", phase).priority(PRIORITY_PHASE),
            StatusField::new("Gen: ", app.generator_algorithm().label()).priority(PRIORITY_GEN),
            StatusField::new("Solve: ", app.solver_algorithm().label()).priority(PRIORITY_SOLVE),
            StatusField::new("Size: ", format!("{maze_width}x{maze_height}"))
                .priority(PRIORITY_SIZE),
            StatusField::new("Steps: ", steps.to_string()).priority(PRIORITY_STEPS),
            StatusField::new("Speed: ", app.speed().to_string()).priority(PRIORITY_SPEED),
            StatusField::activity(activity).priority(PRIORITY_ACTIVITY),
        ],
        usize::from(area.width),
    );

    let controls = Line::from(truncate_text(CONTROLS, usize::from(area.width)));
    let message = message_line(app, usize::from(area.width));

    frame.render_widget(Paragraph::new(vec![status, controls, message]), area);
}

struct StatusField {
    label: &'static str,
    value: String,
    value_style: Style,
    priority: u8,
}

impl StatusField {
    fn new(label: &'static str, value: impl Into<String>) -> Self {
        Self {
            label,
            value: value.into(),
            value_style: Style::default(),
            priority: DEFAULT_PRIORITY,
        }
    }

    fn activity(value: impl Into<String>) -> Self {
        Self {
            label: "",
            value: value.into(),
            value_style: Style::default().add_modifier(Modifier::BOLD),
            priority: DEFAULT_PRIORITY,
        }
    }

    /// Higher-priority fields survive when the status bar cannot fit every
    /// field, independent of their position in the line.
    fn priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    fn width(&self) -> usize {
        self.label.chars().count() + self.value.chars().count()
    }
}

fn status_line(fields: &[StatusField], max_width: usize) -> Line<'static> {
    if max_width == 0 {
        return Line::from("");
    }

    // Decide which fields to show by priority, not position: while the line
    // overflows, drop the lowest-priority field so the most informative ones
    // survive. Among equal priorities the rightmost field drops first, keeping
    // the left-to-right order of the survivors stable.
    let mut kept: Vec<usize> = (0..fields.len()).collect();
    while kept.len() > 1 && rendered_width(fields, &kept) > max_width {
        let drop = kept
            .iter()
            .copied()
            .min_by(|&a, &b| {
                fields[a]
                    .priority
                    .cmp(&fields[b].priority)
                    .then_with(|| b.cmp(&a))
            })
            .expect("kept has at least two entries");
        kept.retain(|&index| index != drop);
    }

    let mut spans = Vec::new();
    let mut used_width = 0;

    for &index in &kept {
        let field = &fields[index];
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

fn rendered_width(fields: &[StatusField], kept: &[usize]) -> usize {
    let field_total: usize = kept.iter().map(|&index| fields[index].width()).sum();
    field_total + kept.len().saturating_sub(1) * STATUS_SEPARATOR.len()
}

fn message_line(app: &App, max_width: usize) -> Line<'static> {
    let base = app.message();
    match app.solved_path_len() {
        Some(path_cells) => {
            let summary = format!("Path: {path_cells} cells");
            let gap = "  ";
            let fits = base.chars().count() + gap.len() + summary.chars().count() <= max_width;
            if fits {
                Line::from(vec![
                    Span::raw(base.to_string()),
                    Span::raw(gap),
                    Span::styled(summary, prominent_style()),
                ])
            } else {
                // Not enough room for both: lead with the prominent summary.
                Line::from(Span::styled(
                    truncate_text(summary, max_width),
                    prominent_style(),
                ))
            }
        }
        None => Line::from(truncate_text(base, max_width)),
    }
}

fn prominent_style() -> Style {
    // Match the final-path color so the readout ties to the highlighted route.
    Style::default()
        .fg(Color::LightYellow)
        .add_modifier(Modifier::BOLD)
}

fn truncate_text(text: impl AsRef<str>, max_width: usize) -> String {
    text.as_ref().chars().take(max_width).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_line_drops_lowest_priority_field_first() {
        // The FIRST field is low priority and the SECOND is high. Only one
        // fits, yet the high-priority field must survive even though it is not
        // first in the line.
        let line = status_line(
            &[
                StatusField::new("Low: ", "x").priority(1),
                StatusField::new("High: ", "y").priority(9),
            ],
            7,
        );

        assert_eq!(line_text(&line), "High: y");
    }

    #[test]
    fn status_line_breaks_ties_by_dropping_rightmost() {
        // Equal priority: the rightmost field drops so the survivors keep a
        // stable left-to-right order.
        let line = status_line(
            &[StatusField::new("A: ", "1"), StatusField::new("B: ", "22")],
            5,
        );

        assert_eq!(line_text(&line), "A: 1");
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
