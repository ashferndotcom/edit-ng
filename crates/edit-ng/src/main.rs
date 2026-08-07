mod app;
mod buffer;
mod dialog;
mod fuzzy;
mod i18n;
mod plugin;
mod syntax;
mod theme;
mod ui;

use app::App;
use argh::FromArgs;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::io::{self, stdout, BufWriter, Write};
use std::panic;
use std::time::Duration;

#[derive(FromArgs, Debug)]
/// edit-ng: Next-Gen Modeless TUI Text Editor with Tree-sitter AST, Fuzzy Finder, and Multi-language support.
struct CliArgs {
    /// optional files to open
    #[argh(positional)]
    files: Vec<String>,

    /// color theme to activate (e.g., monokai, dracula, nord, tokyo-night, catppuccin, gruvbox, solarized, one-dark, github-dark, classic-dos, cyberpunk)
    #[argh(option, short = 't')]
    theme: Option<String>,

    /// interface language code (e.g., en, de, es, fr, hi, ja, ko, zh_hans, ar, ru, pt_br)
    #[argh(option, short = 'l')]
    lang: Option<String>,

    /// print edit-ng version and exit
    #[argh(switch, short = 'v')]
    version: bool,
}

fn main() -> io::Result<()> {
    let args: CliArgs = argh::from_env();

    if args.version {
        println!("edit-ng version {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Set up panic hook to safely restore terminal mode on crashes
    let default_panic = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture);
        default_panic(panic_info);
    }));

    // Initialize raw terminal
    enable_raw_mode()?;
    let mut stdout = BufWriter::with_capacity(65536, stdout());
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let mut app = App::new(&args.files, args.theme, args.lang);

    let mut needs_render = true;

    // Main event loop
    while !app.should_quit {
        if needs_render {
            app.render(&mut stdout)?;
            stdout.flush()?;
            needs_render = false;
        }

        if event::poll(Duration::from_millis(40))? {
            match event::read()? {
                Event::Key(key_event) => {
                    // Only process Press events (ignore Release on Windows/Linux)
                    if key_event.kind == KeyEventKind::Press {
                        app.handle_key_event(key_event);
                        needs_render = true;
                    }
                }
                Event::Resize(w, h) => {
                    app.handle_resize(w, h);
                    needs_render = true;
                }
                Event::Mouse(mouse_event) => {
                    if app.handle_mouse_event(mouse_event) {
                        needs_render = true;
                    }
                }
                _ => {}
            }
        } else if app.check_status_expiration() {
            needs_render = true;
        }
    }

    // Clean up and restore terminal
    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;

    Ok(())
}
