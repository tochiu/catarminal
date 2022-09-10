use crate::{
    // enums,
    render::{
        drag::Dragger,
        space::*,
        shape::BitShape,
        map::{Map, Tile}, world::World
    }
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
    Terminal, widgets::*, layout::{Constraint, Layout, Direction}, style::{Style, Color}, Frame,
};

use rand::Rng;

lazy_static! {
    static ref DOUBLE_UP_BITSHAPE: BitShape = {
        BitShape::new(
            vec![
                0b000000000111000000000000000000000000000011100000000000001110000000000000000000000000000000000000000000000000000111111100,
                0b000000000111000000000000000000000000000011100000000000001110000000000000000000000000000000000000000000000000000111111100,
                0b000111111111000001111100000111000000111011100000000000001110000111111111000000000111000001111011100011111000111100000111,
                0b111111111111000011111100000111000000111011100011111100001110000111000111000000000111000001111011100011111000000000000111,
                0b111000000111001110000011100111000000111011111100000011101110111111111111000000000111000001111011111100001110000000111100,
                0b111000000111001110000011100111000000111011100000000011101110111000000000000000000111000001111011100000001110000111100000,
                0b111111111111001111111111100111111111111011111111111100001110111111111111000000000111111111111011111111111000000000000000,
                0b111111111111000001111100000000111111111011111111111100001110000111111111000000000000111111111011100000000000000111100000,
                0b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000011100000000000000000000000
            ],
            Size2D::new(120, 9)
        )
    };

    static ref DND_NOW_BITSHAPE: BitShape = {
        BitShape::new(
            vec![
                0b111111111111100011111100011111101111111111111000000000011111000011111101111111111111111011111100000011111100111111111111,
                0b111111111111110011111100011111101111111111111100000000011111000011111101111111111111111011111100000011111100111111111111,
                0b111111000111110011111111011111101111110001111100000000011111110011111101111110000111111011111100000011111100111100011111,
                0b111111000111110011111111111111101111110001111100000000011111111111111101111110000111111011111101111111111100000001111111,
                0b111111000111110011111111111111101111110001111100000000011111111111111101111110000111111011111111111111111100000001111110,
                0b111111000111110011111101111111101111110001111100000000011111001111111101111110000111111011111111111111111100000000000000,
                0b111111111111110011111100011111101111111111111100000000011111000011111101111111111111111011111111001111111100000001111000,
                0b111111111111100011111100001111101111111111111000000000011111000001111101111111111111111011111110000111111100000001111000
            ],
            Size2D::new(120, 8)
        )
    };
}

pub fn run(enable_logger: bool) -> Result<(), io::Error> {

    let mut rng = rand::thread_rng();
    let mut world = World::new();

    let mut tiles: Vec<Tile> = Vec::with_capacity(Map::get_tile_capacity());
    
    for _ in 0..tiles.capacity() {
        let roll: u8 = rng.gen_range(2..12);
        tiles.push(Tile::new(if roll > 6 { roll + 1 } else { roll }, rand::random()));
    }
    
    let mut map_dragger = Dragger::new();
    map_dragger.canvas_size = UDim2::from_size2d(Map::get_map_size());
    let map_dragger_id = world.canvas.mount_root(map_dragger).id;
    world.canvas.mount_child(Map::new(tiles), map_dragger_id);

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
                    should_render = should_render || world.input.handle_mouse_input(event, &mut world.canvas);
                }
            }
        }

        if !should_render {
            continue
        }

        terminal.draw(|f| {
            draw_frame(f, &mut world, enable_logger);
        })?;
    }

    Ok(())
}

fn draw_frame<B: Backend>(f: &mut Frame<B>, world: &mut World, draw_logger: bool) {
    

    // Log output
    if draw_logger {
        let rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
            .split(f.size());

        f.render_widget(world.as_widget(), rects[0]);

        let tui_w = TuiLoggerWidget::default()
            .block(
                Block::default()
                    .title("Log output")
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
        f.render_widget(world.as_widget(), f.size());
    }
}