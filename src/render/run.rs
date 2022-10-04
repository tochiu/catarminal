/*
 * run.rs
 * render loop runner + test code hodgepodge :)
 */

use super::{
    screen::Screen,
    drawing::{
        map::{self, Map, Tile, Port},
        game::Game
    }
};

use crate::enums;

use crossterm::{
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui_logger::{TuiLoggerWidget, TuiLoggerLevelOutput};
use std::{io, time::{Duration, Instant}, sync::{Arc, Mutex}, thread, ops::DerefMut, fmt::Write};
use tui::{
    backend::{CrosstermBackend},
    Terminal, widgets::*, 
    layout::{Constraint, Layout, Direction}, 
    style::{Style, Color},
};

use rand::{prelude::Distribution, distributions::Uniform, Rng};

const DEFAULT_REDRAW_DELAY_MS: u64 = 4;

/* this the render loop with some test code for now */
pub fn run(enable_logger: bool) -> Result<(), io::Error> {
    let mut rng = rand::thread_rng();

    let mut tiles: Vec<Tile> = Uniform::from(2..12) // from a uniform distribution from [2, 12)
        .sample_iter(&mut rng) // create an iterator that samples from it
        .take(*map::MAP_TILE_CAPACITY - 1) // sample as many times equal to the map capacity for tiles (minus 1 for desert tile)
        .map(|roll| Tile::new(
            if roll == 7 { 12 } else { roll }, // 7 becomes 12 because 7 is not on a tile its a robber round
            enums::TileResource::Of(rand::random::<enums::Resource>())
        )) 
        .collect();

    // insert desert tile at a random location in the tiles vector
    let desert_tile_index = rng.gen_range(0..=tiles.len());
    tiles.insert(desert_tile_index, Tile::new(7, enums::TileResource::OfDesert));

    let ports: Vec<Port> = enums::PortResource::OfAnyKind // sample_iter takes a self type so I need to do PortResource::OfAnyKind instead of just PortResource
        .sample_iter(&mut rng) // create an iterator that takes samples of PortResource
        .take(*map::MAP_PORT_CAPACITY) // sample as many times equal to the map capacity for ports
        .enumerate()
        .map(|(i, resource)| Port::new(i, resource)) // we want ports containing these resources
        .collect();

    let game_screen_resource = Arc::new(Mutex::new(Screen::new(Game::new(Map::new(tiles, ports)))));

    // { // test robber animation
    //     let game_screen_mutex = Arc::clone(&game_screen_resource);
    //     thread::spawn(move || {
    //         let mut rng = rand::thread_rng();
    //         let mut last_num = desert_tile_index;
    //         loop {
    //             thread::sleep(Duration::from_secs(1));
    //             let mut guard = game_screen_mutex.lock().unwrap();
    //             let game_screen = guard.deref_mut();
                
    //             let mut num = rng.gen_range(0..*map::MAP_TILE_CAPACITY - 1);
    //             if num == last_num {
    //                 num = *map::MAP_TILE_CAPACITY - 1;
    //             }

    //             game_screen.root.map_dragger.drawing.move_robber(num, &mut game_screen.service.animation);
    //             last_num = num;
    //         }
    //     });
    // }

    { // test road + building placement animations
        let game_screen_mutex = Arc::clone(&game_screen_resource);
        thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let road_plots = map::MAP_GRAPH.plot_edges
                .iter()
                .enumerate()
                .map(|(from_plot, edges)| {
                    edges
                        .iter()
                        .filter_map(move |&to_plot| if from_plot < to_plot { Some((from_plot, to_plot)) } else { None })
                })
                .flatten();
            
            for tile in 0..map::MAP_GRAPH.tile_anchor_points.len() {
                thread::sleep(Duration::from_millis(250));
                let mut guard = game_screen_mutex.lock().unwrap();
                let game_screen = guard.deref_mut();

                game_screen.root.map_dragger.drawing.place_tile(
                    tile,
                    &mut game_screen.service.animation
                );
            }

            {
                let game_screen_mutex = Arc::clone(&game_screen_mutex);
                thread::spawn(move || {
                    for port in 0..map::MAP_GRAPH.port_points.len() {
                        thread::sleep(Duration::from_millis(250));
                        let mut guard = game_screen_mutex.lock().unwrap();
                        let game_screen = guard.deref_mut();
        
                        game_screen.root.map_dragger.drawing.show_port(port, &mut game_screen.service.animation);
                    }
                });
            }

            for (plot_a, plot_b) in road_plots {
                thread::sleep(Duration::from_millis(100));
                let mut guard = game_screen_mutex.lock().unwrap();
                let game_screen = guard.deref_mut();

                game_screen.root.map_dragger.drawing.place_road(
                    plot_a, plot_b, 
                    Style::default().bg(Color::Rgb(
                        rng.gen_range(0..=225), 
                        rng.gen_range(0..=225), 
                        rng.gen_range(0..=225)
                    )), 
                    &mut game_screen.service.animation
                );
            }

            for plot in 0..map::MAP_GRAPH.plot_points.len() {
                thread::sleep(Duration::from_millis(100));
                let mut guard = game_screen_mutex.lock().unwrap();
                let game_screen = guard.deref_mut();

                game_screen.root.map_dragger.drawing.place_building(
                    plot,
                    enums::Building::Settlement.sample(&mut rng),
                    Style::default().bg(Color::Rgb(
                        rng.gen_range(0..=225), 
                        rng.gen_range(0..=225), 
                        rng.gen_range(0..=225)
                    )), 
                    &mut game_screen.service.animation
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
    //let mut frame_number: u128 = 0;
    let mut delay_ms = 0;
    let mut times_str = String::new();
    let mut flush_time = 0;

    loop {
        let mut run_step_start = Instant::now();
        let mut should_render = false;
        let mut maybe_mouse_event: Option<MouseEvent> = None;
        let mut input_poll_time = 0;

        if poll(Duration::from_millis(delay_ms))? {
            input_poll_time = run_step_start.elapsed().as_millis();
            run_step_start = Instant::now();
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

        delay_ms = if game_screen.service.animation.count() > 0 { 0 } else { DEFAULT_REDRAW_DELAY_MS };
        should_render = should_render || game_screen.service.animation.count() > 0;

        if let Some(mouse_event) = maybe_mouse_event {
            log::info!("mouse event: {:?}", mouse_event);
            // call on separate line because we dont want short-circuiting to prevent mouse input handler from running
            let event_requires_rerender = game_screen.service.input.handle_mouse_input(mouse_event, &mut game_screen.root); 
            should_render = should_render || event_requires_rerender;
        } 

        if !should_render {
            continue
        }

        // log::info!("anims to render: {}", game_screen.service.animation.count());
        // if game_screen.service.animation.count() == 1 {
        //     log::info!("alpha: {:?}", game_screen.service.animation.animations.values().next().unwrap().duration)
        // }

        //log::info!("drawing frame #{:?}", frame_number);
        //frame_number += 1;
        let mut frame_draw_time = 0;
        let total_start = Instant::now();
        let mut flush_start = Instant::now();
        terminal.draw(|f| {
            let (times_area, area) = {
                let areas = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(1), Constraint::Length(f.size().y.saturating_sub(1))])
                    .split(f.size());
                (areas[0], areas[1])
            };
            
            if enable_logger {
                /* divide screen for the logger and game */
                let rects = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                    .split(area);
                
                /* draw the game screen */
                let frame_start = Instant::now();
                f.render_stateful_widget(game_screen.as_stateful_widget(), rects[0], &mut ());
                frame_draw_time = frame_start.elapsed().as_millis();
                
                let tui_w = TuiLoggerWidget::default()
                    .block(
                        Block::default()
                            .title(" Log ")
                            .border_style(Style::default().fg(Color::White).bg(Color::Black))
                            .borders(Borders::ALL),
                    )
                    .output_separator('|')
                    .output_timestamp(Some("%H:%M:%S%.3f".to_string()))
                    .output_level(Some(TuiLoggerLevelOutput::Abbreviated))
                    .output_target(false)
                    .output_file(false)
                    .output_line(false)
                    .style_error(Style::default().fg(Color::Red))
                    .style_debug(Style::default().fg(Color::Cyan))
                    .style_warn(Style::default().fg(Color::Yellow))
                    .style_trace(Style::default().fg(Color::White))
                    .style_info(Style::default().fg(Color::Green));
                
                /* draw the logger */
                f.render_widget(tui_w, rects[1]);
            } else {
                /* draw the game */
                let frame_start = Instant::now();
                f.render_stateful_widget(game_screen.as_stateful_widget(), area, &mut ());
                frame_draw_time = frame_start.elapsed().as_millis();
            }

            let total_draw_time = total_start.elapsed().as_millis();
            let run_step_time = run_step_start.elapsed().as_millis();
            times_str.clear();
            write!(times_str, 
                "frame draw time: {:02} ms | total draw time: {:02} ms | run step time: {:02} ms | last flush time: {:02} ms | input poll time: {:02} ms", 
                frame_draw_time, total_draw_time, run_step_time, flush_time, input_poll_time
            ).unwrap();
            /* draw the frame times */
            f.render_stateful_widget(StringLineWidget, times_area, &mut times_str);
            flush_start = Instant::now();
        })?;
        flush_time = flush_start.elapsed().as_millis();
    }

    Ok(())
}

struct StringLineWidget;
impl StatefulWidget for StringLineWidget {
    type State = String;
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer, state: &mut Self::State) {
        buf.set_string(area.x, area.y, state, Style::default())
    }
}