use crate::render::{draw::*, space::*, drawing::counter::*};

use tui::style::{Style, Color, Modifier};
use unicode_width::UnicodeWidthStr;

/* PlayerList */

#[derive(Debug)]
pub struct PlayerList {
    pub frames: Vec<PlayerFrame>,
    pub layout: DrawLayout
}

impl PlayerList {
    pub fn new(mut frames: Vec<PlayerFrame>, layout: DrawLayout) -> Self {
        for (i, frame) in frames.iter_mut().enumerate() {
            frame.layout
                .set_position(UDim2::new(0.5, 0, 1.0, -(i as i16)*(PLAYER_FRAME_SIZE.y.offset as i16)))
                .set_anchor(Float2D::new(0.5, 1.0));
        }
        PlayerList { frames, layout }
    }
}

impl Layoutable for PlayerList {
    fn layout_ref(&self) -> &DrawLayout { &self.layout }
    fn layout_mut(&mut self) -> &mut DrawLayout { &mut self.layout }
}

impl StatefulDrawable for PlayerList {
    type State = [PlayerFrameState];

    fn stateful_draw(&self, ctx: &mut DrawContext, state: &Self::State) {
        ctx.draw_stateful_children(self.frames.as_slice(), state);
    }
}

pub const PLAYER_FRAME_SIZE: UDim2 = UDim2::new(1.0, 0, 0.0, 5);

pub struct PlayerFrameState {
    pub victory_point_count: u8,
    pub resource_card_count: u8,
    pub development_card_count: u8,
    pub largest_army_count: u8,
    pub longest_road_count: u8
}

#[derive(Debug)]
pub struct PlayerFrame {
    player_name: String,
    player_color: Color,
    army_counter: SymbolCounter,
    road_counter: SymbolCounter,
    resource_counter: SymbolCounter,
    development_counter: SymbolCounter,
    
    pub layout: DrawLayout
}

/* PlayerFrame */

lazy_static! {
    static ref CARD_SYMBOL_STYLE: Style = Style::default().fg(Color::White);
    static ref CARD_TEXT_STYLE: Style = Style::default().fg(Color::White).add_modifier(Modifier::BOLD);
}

impl PlayerFrame {
    pub fn new(player_name: String, player_color: Color, mut layout: DrawLayout) -> Self {
        layout.set_size(PLAYER_FRAME_SIZE);
        let mut frame = PlayerFrame {
            layout,
            player_name,
            player_color,
            resource_counter: SymbolCounter::new(
                String::from("??"), 
                SymbolCounterKind::Card, 
                SymbolCounterStyle {
                    symbol: *CARD_SYMBOL_STYLE, 
                    text: *CARD_TEXT_STYLE, 
                    bkg: Style::default().bg(Color::LightBlue)
                }, 
                DrawLayout::default()
            ),
            development_counter: SymbolCounter::new(
                String::from("🔨"),
                SymbolCounterKind::Card, 
                SymbolCounterStyle {
                    symbol: *CARD_SYMBOL_STYLE, 
                    text: *CARD_TEXT_STYLE, 
                    bkg: Style::default().bg(Color::Magenta)
                }, 
                DrawLayout::default()
            ),
            army_counter: SymbolCounter::new(
                String::from("⚔️"),
                SymbolCounterKind::Default, 
                SymbolCounterStyle {
                    symbol: *CARD_SYMBOL_STYLE, 
                    text: Style::default().fg(Color::White).add_modifier(Modifier::BOLD), 
                    bkg: Style::default()
                }, 
                DrawLayout::default()
            ),
            road_counter: SymbolCounter::new(
                String::from("🛤️"),
                SymbolCounterKind::Default,
                SymbolCounterStyle {
                    symbol: *CARD_SYMBOL_STYLE, 
                    text: Style::default().fg(Color::White).add_modifier(Modifier::BOLD), 
                    bkg: Style::default()
                }, 
                DrawLayout::default()
            )
        };

        let order = &mut [
            &mut frame.resource_counter,
            &mut frame.development_counter,
            &mut frame.army_counter,
            &mut frame.road_counter
        ];
        let order_length = order.len();
        for (i, card_counter) in order.iter_mut().enumerate() {
            card_counter.layout.set_position(UDim2::new(0.5, (4 + 2)*i as i16 - ((4*order_length + 2*(order_length - 1))/2) as i16, 0.0, 2));
        }

        frame
    }
}

impl Layoutable for PlayerFrame {
    fn layout_ref(&self) -> &DrawLayout { &self.layout }
    fn layout_mut(&mut self) -> &mut DrawLayout { &mut self.layout }
}

impl StatefulDrawable for PlayerFrame {
    type State = PlayerFrameState;

    fn stateful_draw(&self, ctx: &mut DrawContext, state: &Self::State) {
        ctx.draw_stateful_child(&self.army_counter, &state.largest_army_count);
        ctx.draw_stateful_child(&self.road_counter, &state.longest_road_count);
        ctx.draw_stateful_child(&self.resource_counter, &state.resource_card_count);
        ctx.draw_stateful_child(&self.development_counter, &state.development_card_count);

        let vp_string = &format!("🏆: {:0width$}", state.victory_point_count.min(99), width = 2 as usize);

        ctx.draw_unicode_line(
            &vp_string,
            Point2D::new((ctx.absolute_layout_space.size.x as i16 - vp_string.width() as i16)/2, 1),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        );
        ctx.draw_unicode_line(
            &tui::symbols::line::NORMAL.horizontal.repeat(ctx.absolute_layout_space.size.x as usize),
            Point2D::new(0, 0),
            Style::default().fg(Color::White)
        );
        ctx.draw_string_line(
            &format!(" {} ", self.player_name),
            Point2D::new((ctx.absolute_layout_space.size.x as i16 - self.player_name.len() as i16)/2 - 1, 0),
            Style::default()
                .fg(self.player_color)
                .add_modifier(Modifier::BOLD)
        );
    }
}

