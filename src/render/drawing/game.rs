use super::{players::{self, *}, map::{self, Map}, drag::Dragger};

use crate::render::prelude::*;

use tui::{
    layout::*, 
    style::{Color, Style}, 
    widgets::{Block, Borders}
};

#[derive(Debug, Default)]
struct GameRegions {
    map: AbsoluteSpace,
    events: AbsoluteSpace,
    chat: AbsoluteSpace,
    players: AbsoluteSpace
}

#[derive(Debug)]
pub struct Game {
    mount: Mount,
    layout: DrawLayout,
    players: PlayerList,
    regions: GameRegions,    

    pub map_dragger: Dragger<Map>,
}

impl Game {
    pub fn new(map: Map) -> Self {
        Game {
            mount: Mount::default(),
            layout: DrawLayout::FULL,
            map_dragger: Dragger::new(map, Style::default().bg(map::MAP_OCEAN_COLOR)),
            regions: GameRegions::default(),
            players: PlayerList::new(
                vec![
                    PlayerFrame::new(
                        String::from("Andre"), 
                        Color::Red, 
                        DrawLayout::default()
                    ),
                    PlayerFrame::new(
                        String::from("Unbreakable Patches"), 
                        Color::Green, 
                        DrawLayout::default()
                    ),
                    PlayerFrame::new(
                        String::from("Seigward"), 
                        Color::Blue, 
                        DrawLayout::default()
                    )
                ], 
                DrawLayout::FULL
            )
        }
    }
}

impl Layoutable for Game {
    fn layout_ref(&self) -> &DrawLayout { &self.layout }
    fn layout_mut(&mut self) -> &mut DrawLayout { &mut self.layout }
}

impl StatefulDrawable for Game {
    type State = ();
    fn stateful_draw(&self, ctx: &mut DrawContext, state: &Self::State) {
        ctx.draw_widget(
            Block::default()
                .title(" Map ")
                .border_style(Style::default().fg(Color::White).bg(Color::Black))
                .borders(Borders::ALL), 
            self.regions.map
        );
        ctx.draw_widget(
            Block::default()
                .title(" Events ")
                .border_style(Style::default().fg(Color::White).bg(Color::Black))
                .borders(Borders::ALL), 
            self.regions.events
        );
        ctx.draw_widget(
            Block::default()
                .title(" Chat ")
                .border_style(Style::default().fg(Color::White).bg(Color::Black))
                .borders(Borders::ALL), 
            self.regions.chat
        );
        ctx.draw_widget(
            Block::default()
                .title(" Players ")
                .border_style(Style::default().fg(Color::White).bg(Color::Black))
                .borders(Borders::ALL), 
            self.regions.players
        );
        
        ctx.draw_stateful_child(&self.map_dragger, state);
        ctx.draw_stateful_child(&self.players, &[
            // TEST DATA
            PlayerFrameState {
                victory_point_count: 9,
                largest_army_count: 2,
                longest_road_count: 3,
                resource_card_count: 7,
                development_card_count: 0
            },
            PlayerFrameState {
                victory_point_count: 10,
                largest_army_count: 5,
                longest_road_count: 8,
                resource_card_count: 22,
                development_card_count: 6
            },
            PlayerFrameState {
                victory_point_count: 4,
                largest_army_count: 0,
                longest_road_count: 0,
                resource_card_count: 0,
                development_card_count: 0
            }
        ]);
    }
}

impl MountableLayout for Game {
    fn mount_ref(&self) -> &Mount { &self.mount }
    fn mount_mut(&mut self) -> &mut Mount { &mut self.mount }
    fn child_ref(&self, i: usize) -> Option<&dyn MountableLayout> { 
        match i {
            0 => Some(self.map_dragger.as_trait_ref()),
            _ => None
        } 
    }
    fn child_mut(&mut self, i: usize) -> Option<&mut dyn MountableLayout> { 
        match i {
            0 => Some(self.map_dragger.as_trait_mut()),
            _ => None
        } 
    }

    fn relayout(&mut self, ctx: &mut LayoutContext) {
        let space = ctx.get_absolute_size_of(self).to_rect();
        let rects = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(space.width.saturating_sub(39)), Constraint::Min(39)].as_ref())
            .split(space);

        let player_list_height = (self.players.frames.len() as u16)*(players::PLAYER_FRAME_SIZE.y.offset as u16) + 2;
        let right_pane_rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(rects[1].height.saturating_sub(player_list_height)), Constraint::Min(player_list_height)].as_ref())
            .split(rects[1]);
        let chat_event_rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(right_pane_rects[0]);

        self.regions.map = AbsoluteSpace::from_rect(rects[0]);
        self.regions.events = AbsoluteSpace::from_rect(chat_event_rects[0]);
        self.regions.chat = AbsoluteSpace::from_rect(chat_event_rects[1]);
        self.regions.players = AbsoluteSpace::from_rect(right_pane_rects[1]);

        let map_space = AbsoluteSpace::from_rect(
            Block::default()
                .borders(Borders::ALL)
                .inner(self.regions.map.to_rect())
        );
        self.map_dragger.layout.set_size(UDim2::from_size2d(map_space.size));
        self.map_dragger.layout.set_position(UDim2::from_point2d(map_space.position));

        let players_space = AbsoluteSpace::from_rect(
            Block::default()
                .borders(Borders::ALL)
                .inner(self.regions.players.to_rect())
        );
        self.players.layout.set_position(UDim2::from_point2d(players_space.position));
        self.players.layout.set_size(UDim2::from_size2d(players_space.size));
        
        ctx.relayout_children_of(self.as_trait_mut());
    }
}