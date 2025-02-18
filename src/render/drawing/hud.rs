use crate::render::{draw::*, space::*, drawing::counter::*};

use tui::style::{Style, Color, Modifier};
use unicode_width::UnicodeWidthStr;

/* PlayerList */

#[derive(Debug)]
pub struct ActionBar {
    trade: ActionButton,
    place_road: ActionButton,
    place_settlement: ActionButton,
    place_city: ActionButton,
    draw_dev_card: ActionButton,
    end_turn: ActionButton,

    pub layout: DrawLayout
}

impl PlayerList {
    pub fn new(mut frames: Vec<ActionButton>, layout: DrawLayout) -> Self {
        for (i, frame) in frames.iter_mut().enumerate() {
            frame.layout
                .set_position(UDim2::new(0.5, 0, 1.0, -(i as i16)*(ACTION_BUTTON_SIZE.y.offset as i16)))
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

pub const ACTION_BUTTON_SIZE: UDim2 = UDim2::new(0.0, 4, 1.0, 0);

pub struct PlayerFrameState {
    pub victory_point_count: u8,
    pub resource_card_count: u8,
    pub development_card_count: u8,
    pub largest_army_count: u8,
    pub longest_road_count: u8
}

#[derive(Debug)]
pub struct ActionButton {
    title: String,
    pub layout: DrawLayout
}

/* PlayerFrame */

impl ActionButton {
    pub fn new(title: String, mut layout: DrawLayout) -> Self {
        layout.set_size(ACTION_BUTTON_SIZE);
        ActionButton {
            title,
            layout
        }
    }
}

impl Layoutable for ActionButton {
    fn layout_ref(&self) -> &DrawLayout { &self.layout }
    fn layout_mut(&mut self) -> &mut DrawLayout { &mut self.layout }
}

impl Drawable for ActionButton {
    fn draw(&self, ctx: &mut DrawContext) {
        ctx.draw_string_line(
            &format!(" {} ", self.title),
            Point2D::new((ctx.absolute_layout_space.size.x as i16 - self.title.len() as i16)/2 - 1, 0),
            Style::default()
                .add_modifier(Modifier::BOLD)
        );
    }
}

// impl StatefulDrawable for ActionButton {
//     type State = PlayerFrameState;

//     fn stateful_draw(&self, ctx: &mut DrawContext, state: &Self::State) {
//         ctx.draw_stateful_child(&self.army_counter, &state.largest_army_count);
//         ctx.draw_stateful_child(&self.road_counter, &state.longest_road_count);
//         ctx.draw_stateful_child(&self.resource_counter, &state.resource_card_count);
//         ctx.draw_stateful_child(&self.development_counter, &state.development_card_count);

//         let vp_string = &format!("🏆: {:0width$}", state.victory_point_count.min(99), width = 2 as usize);

//         ctx.draw_unicode_line(
//             &vp_string,
//             Point2D::new((ctx.absolute_layout_space.size.x as i16 - vp_string.width() as i16)/2, 1),
//             Style::default()
//                 .fg(Color::White)
//                 .add_modifier(Modifier::BOLD)
//         );
//         ctx.draw_unicode_line(
//             &tui::symbols::line::NORMAL.horizontal.repeat(ctx.absolute_layout_space.size.x as usize),
//             Point2D::new(0, 0),
//             Style::default().fg(Color::White)
//         );
//         ctx.draw_string_line(
//             &format!(" {} ", self.player_name),
//             Point2D::new((ctx.absolute_layout_space.size.x as i16 - self.player_name.len() as i16)/2 - 1, 0),
//             Style::default()
//                 .fg(self.player_color)
//                 .add_modifier(Modifier::BOLD)
//         );
//     }
// }

