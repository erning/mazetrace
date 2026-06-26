use std::io;
use std::time::Duration;

use clap::Parser;
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use mazetrace::app::App;
use mazetrace::config::Config;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

fn main() {
    if let Err(err) = run() {
        eprintln!("mazetrace: {err}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let config = Config::parse();
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = run_app(&mut terminal, config);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    config: Config,
) -> io::Result<()> {
    let size = terminal.size()?;
    let mut app = App::new(config, size.width, size.height);

    loop {
        terminal.draw(|frame| mazetrace::ui::draw(frame, &app))?;

        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if app.handle_key(key) {
                        return Ok(());
                    }
                }
                Event::Resize(width, height) => app.handle_resize(width, height),
                _ => {}
            }
        }

        app.tick();
    }
}
