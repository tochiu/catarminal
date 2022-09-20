use crate::enums::PortResource;

use super::{
    map::{self, Map, Tile, Port},
    game::Game, 
    screen::Screen, 
    draw::NoDrawState
};

use crossterm::{
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui_logger::{TuiLoggerWidget, TuiLoggerLevelOutput};
use std::{io, time::Duration};
use tui::{
    backend::{CrosstermBackend},
    Terminal, widgets::*, layout::{Constraint, Layout, Direction}, style::{Style, Color},
};

use rand::{prelude::Distribution, distributions::Uniform};

pub fn run(enable_logger: bool) -> Result<(), io::Error> {
    let mut rng = rand::thread_rng();

    let tiles: Vec<Tile> = Uniform::from(2..12)
        .sample_iter(&mut rng)
        .take(*map::MAP_TILE_CAPACITY)
        .map(|roll| Tile::new(if roll > 6 { roll + 1 } else { roll }, rand::random()))
        .collect();

    let ports: Vec<Port> = PortResource::Any
        .sample_iter(&mut rng)
        .take(*map::MAP_PORT_CAPACITY)
        .map(|resource| Port::new(resource))
        .collect();

    let mut screen = Screen::new(Game::new(Map::new(tiles, ports)));

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture, crossterm::terminal::DisableLineWrap)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        let mut should_render = false;
        if poll(Duration::from_millis(16))? {
            match read()? {
                Event::Resize(_, _) => should_render = true,
                Event::Key(key) => {
                    match key.code {
                        KeyCode::Esc => {
                            // restore terminal
                            disable_raw_mode()?;
                            execute!(
                                terminal.backend_mut(),
                                LeaveAlternateScreen,
                                DisableMouseCapture
                            )?;
                            terminal.show_cursor()?;

                            break
                        },
                        _ => ()
                    }
                },
                Event::Mouse(event) => {
                    log::info!("mouse event: {:?}", event);
                    // call on separate line because we dont want short-circuiting to prevent mouse input handler from running
                    let input_requires_rerender = screen.input.handle_mouse_input(event, &mut screen.root); 
                    should_render = should_render || input_requires_rerender;
                }
            }
        }

        if !should_render {
            continue
        }

        terminal.draw(|f| {
            if enable_logger {
                let rects = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                    .split(f.size());
                
                f.render_stateful_widget(screen.as_widget(), rects[0], &mut NoDrawState);
                
                let tui_w = TuiLoggerWidget::default()
                    .block(
                        Block::default()
                            .title(" Log ")
                            .border_style(Style::default().fg(Color::White).bg(Color::Black))
                            .borders(Borders::ALL),
                    )
                    .output_separator('|')
                    .output_timestamp(Some("%F %H:%M:%S%.3f ".to_string()))
                    .output_level(Some(TuiLoggerLevelOutput::Long))
                    .output_target(false)
                    .output_file(false)
                    .output_line(false)
                    .style_error(Style::default().fg(Color::Red))
                    .style_debug(Style::default().fg(Color::Cyan))
                    .style_warn(Style::default().fg(Color::Yellow))
                    .style_trace(Style::default().fg(Color::White))
                    .style_info(Style::default().fg(Color::Green));

                f.render_widget(tui_w, rects[1]);
            } else {
                f.render_stateful_widget(screen.as_widget(), f.size(), &mut NoDrawState);
            }
        })?;
    }

    Ok(())
}