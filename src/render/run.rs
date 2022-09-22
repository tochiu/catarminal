use super::{
    drawing::{
        map::{self, Map, Tile, Port},
        game::Game
    }, 
    screen::Screen, 
    draw::NoDrawState
};

use crate::enums;

use crossterm::{
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui_logger::{TuiLoggerWidget, TuiLoggerLevelOutput};
use std::{io, time::Duration, sync::{Arc, Mutex}, thread, ops::DerefMut};
use tui::{
    backend::{CrosstermBackend},
    Terminal, widgets::*, 
    layout::{Constraint, Layout, Direction}, 
    style::{Style, Color},
};

use rand::{prelude::Distribution, distributions::Uniform, Rng};

const DEFAULT_REDRAW_DELAY_MS: u64 = 4;

pub fn run(enable_logger: bool) -> Result<(), io::Error> {
    let mut rng = rand::thread_rng();

    let mut tiles: Vec<Tile> = Uniform::from(2..12) // from a uniform distribution from 2-12
        .sample_iter(&mut rng) // create an iterator that samples from it
        .take(*map::MAP_TILE_CAPACITY - 1) // sample as many times equal to the map capacity for tiles (minus 1 for desert tile)
        .map(|roll| Tile::new(
            if roll == 7 { 12 } else { roll }, // 7 becomes 12 because 7 is not on a tile its a robber round
            enums::TileResource::Of(rand::random::<enums::Resource>())
        )) 
        .collect();

    // insert desert tile at a random location in the tiles vector
    tiles.insert(rng.gen_range(0..=tiles.len()), Tile::new(7, enums::TileResource::OfDesert));

    let ports: Vec<Port> = enums::PortResource::OfAnyKind // sample_iter takes a self type so I need to do PortResource::OfAnyKind instead of just PortResource
        .sample_iter(&mut rng) // create an iterator that takes samples of PortResource
        .take(*map::MAP_PORT_CAPACITY) // sample as many times equal to the map capacity for ports
        .map(|resource| Port::new(resource)) // we want ports containing these resources
        .collect();

    let game_screen_resource = Arc::new(Mutex::new(Screen::new(Game::new(Map::new(tiles, ports)))));

    { // test robber animation
        let game_screen_mutex = Arc::clone(&game_screen_resource);
        thread::spawn(move || {
            let mut rng = rand::thread_rng();
            loop {
                thread::sleep(Duration::from_secs(1));
                let mut guard = game_screen_mutex.lock().unwrap();
                let game_screen = guard.deref_mut();
                
                game_screen.root.map_dragger.drawing.move_robber(
                    rng.gen_range(0..*map::MAP_PORT_CAPACITY), 
                    &mut game_screen.animation
                );
            }
        });
    }

    let game_screen_mutex = Arc::clone(&game_screen_resource);

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut frame_number: u128 = 0;
    let mut delay_ms = 0;

    loop {
        let mut should_render = false;
        let mut maybe_mouse_event: Option<MouseEvent> = None;

        if poll(Duration::from_millis(delay_ms))? {
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
                Event::Mouse(event) => maybe_mouse_event = Some(event)
            }
        }

        // game a lock of the game_screen here
        //let mut game_screen = game_screen_mutex.lock().unwrap();
        let mut guard = game_screen_mutex.lock().unwrap();
        let game_screen = guard.deref_mut();

        delay_ms = if game_screen.animation.contains_any() { 0 } else { DEFAULT_REDRAW_DELAY_MS };
        should_render = should_render || game_screen.animation.contains_any();

        if let Some(mouse_event) = maybe_mouse_event {
            log::info!("mouse event: {:?}", mouse_event);
            // call on separate line because we dont want short-circuiting to prevent mouse input handler from running
            let event_requires_rerender = game_screen.input.handle_mouse_input(mouse_event, &mut game_screen.root); 
            should_render = should_render || event_requires_rerender;
        } 

        if !should_render {
            continue
        }

        log::info!("drawing frame #{:?}", frame_number);
        frame_number += 1;

        terminal.draw(|f| {
            if enable_logger {
                let rects = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                    .split(f.size());
                
                f.render_stateful_widget(game_screen.as_widget(), rects[0], &mut NoDrawState);
                
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
                f.render_stateful_widget(game_screen.as_widget(), f.size(), &mut NoDrawState);
            }
        })?;
    }

    Ok(())
}