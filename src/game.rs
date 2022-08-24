use crate::{
    // enums,
    render::{
        space::*,
        shape::BitShape,
        draw::Drawing,
        map::{Map, Tile}
    }
};

use crossterm::{
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, time::Duration};
use tui::{
    backend::CrosstermBackend,
    Terminal,
};

use rand::Rng;

use std::fs::File;
use std::io::prelude::*;

pub fn run() -> Result<(), io::Error> {
    let mut rng = rand::thread_rng();
    let double_up = BitShape::new(
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
    );

    let dnd_now = BitShape::new(
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
    );
    
    let mut file = File::open("./assets/map.txt").expect("Cannot open the file");
    let mut file_str = String::new();
    file.read_to_string(&mut file_str).expect("Cannot read the file");

    let mut map_drawing = Drawing::a(Map::from(&file_str, &dnd_now));
    let mut tiles: Vec<Drawing<Tile>> = Vec::with_capacity(map_drawing.pencil.tile_capacity);
    
    for _ in 0..map_drawing.pencil.tile_capacity {
        let roll: u8 = rng.gen_range(2..12);
        tiles.push(Drawing::a(Tile::from(if roll > 6 { roll + 1 } else { roll }, rand::random())));
    }

    map_drawing.pencil.set_tiles(tiles);

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
                Event::Resize(_, _) => should_render = true, //terminal.resize(Rect::new(0, 0, x, y))?,
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
                    match event.kind {
                        MouseEventKind::Moved => {
                            should_render = true;
                            map_drawing.pencil.set_cursor_pos(UDim2::from_offset(event.column as i16, event.row as i16));
                        },
                        _ => ()
                    }
                }
            }
        }

        if !should_render {
            continue
        }

        // let dx: i16 = rng.gen_range(-10..11);
        // let dy: i16 = rng.gen_range(-10..11);
        // map_drawing.pencil.set_four_pos(UDim2::new(0.5, dx, 0.5, dy));

        terminal.draw(|f| {
            f.render_widget(map_drawing.to_widget(), f.size());
        })?;
    }

    Ok(())
}
