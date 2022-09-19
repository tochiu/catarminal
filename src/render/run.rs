use super::{
    map::{self, Map, Tile}, 
    world::World
};

use crossterm::{
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui_logger::{TuiLoggerWidget, TuiLoggerLevelOutput};
use std::{io, time::Duration};
use tui::{
    backend::{CrosstermBackend, Backend},
    Terminal, widgets::*, layout::{Constraint, Layout, Direction, Rect}, style::{Style, Color}, Frame,
};

use rand::Rng;

pub fn run(enable_logger: bool) -> Result<(), io::Error> {
    let mut rng = rand::thread_rng();
    let mut tiles: Vec<Tile> = Vec::with_capacity(*map::MAP_TILE_CAPACITY);
    
    for _ in 0..tiles.capacity() {
        let roll: u8 = rng.gen_range(2..12);
        tiles.push(Tile::new(if roll > 6 { roll + 1 } else { roll }, rand::random()));
    }

    let mut world = World::new(Map::new(tiles));

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
                    let input_requires_rerender = world.input.handle_mouse_input(event, &mut world.root); 
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
                
                draw_frame(f, rects[0], &mut world);
                
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
                draw_frame(f, f.size(), &mut world);
            }
        })?;
    }

    Ok(())
}

fn draw_frame<B: Backend>(f: &mut Frame<B>, space: Rect, world: &mut World) {
    // TODO: refactor this garbage since tui's built in layout system kinda blows

    let rects = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(space.width.saturating_sub(39)), Constraint::Min(39)].as_ref())
        .split(space);
    
    let event_log = Block::default()
        .title(" Events ")
        .border_style(Style::default().fg(Color::White).bg(Color::Black))
        .borders(Borders::ALL);
    let chat = Block::default()
        .title(" Chat ")
        .border_style(Style::default().fg(Color::White).bg(Color::Black))
        .borders(Borders::ALL);
    let players = Block::default()
        .title(" Players ")
        .border_style(Style::default().fg(Color::White).bg(Color::Black))
        .borders(Borders::ALL);
    
    let map_block = Block::default()
        .title(" Map ")
        .border_style(Style::default().fg(Color::White).bg(Color::Black))
        .borders(Borders::ALL);
    let map_rect = map_block.inner(rects[0]);

    let right_pane_rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(rects[1].height.saturating_sub(5*4)), Constraint::Min(5*4)].as_ref())
        .split(rects[1]);
    let chat_event_rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(right_pane_rects[0]);

    
    f.render_widget(map_block, rects[0]);
    f.render_widget(world.as_widget(), map_rect);

    f.render_widget(event_log, chat_event_rects[0]);
    f.render_widget(chat, chat_event_rects[1]);
    f.render_widget(players, right_pane_rects[1]);
}