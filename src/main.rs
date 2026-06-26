use std::io;
use std::time::Duration;

use clap::Parser;
use crossterm::cursor::Show;
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
    if config.uses_deprecated_algorithm_alias() {
        eprintln!("mazetrace: --algorithm is deprecated; use --solver instead.");
    }

    let _terminal_session = TerminalSession::enter()?;

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    run_app(&mut terminal, config)
}

struct TerminalSession {
    raw_mode_enabled: bool,
    alternate_screen_entered: bool,
}

impl TerminalSession {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;

        let mut session = Self {
            raw_mode_enabled: true,
            alternate_screen_entered: false,
        };
        execute!(io::stdout(), EnterAlternateScreen)?;
        session.alternate_screen_entered = true;

        Ok(session)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        if self.alternate_screen_entered {
            let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
        }

        if self.raw_mode_enabled {
            let _ = disable_raw_mode();
        }
    }
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
