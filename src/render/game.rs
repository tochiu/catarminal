use super::{
    drag::Dragger, 
    map::Map, 
    draw::*,
    mount::*,
    world::*, 
    space::*
};

use tui::{
    layout::*, 
    style::{Color, Style}, 
    widgets::{Block, Borders}
};

#[derive(Debug, Default)]
struct GameRegions {
    map: Rect,
    events: Rect,
    chat: Rect,
    players: Rect
}

#[derive(Debug)]
pub struct Game {
    mount: Mount,
    layout: DrawLayout,
    map_dragger: Dragger<Map>,
    regions: GameRegions,
}

impl Game {
    pub fn new(map: Map) -> Self {
        Game {
            mount: Mount::default(),
            layout: DrawLayout::FULL,
            map_dragger: Dragger::new(map),
            regions: GameRegions::default()
        }
    }
}

impl Layoutable for Game {
    fn layout_ref(&self) -> &DrawLayout {
        &self.layout
    }
}

impl StatefulDrawable for Game {
    type State = NoDrawState;
    fn stateful_draw(&self, mut area: WorldArea, state: &Self::State) {
        area.draw_widget(
            Block::default()
                .title(" Map ")
                .border_style(Style::default().fg(Color::White).bg(Color::Black))
                .borders(Borders::ALL), 
            self.regions.map
        );
        area.draw_widget(
            Block::default()
                .title(" Events ")
                .border_style(Style::default().fg(Color::White).bg(Color::Black))
                .borders(Borders::ALL), 
            self.regions.events
        );
        area.draw_widget(
            Block::default()
                .title(" Chat ")
                .border_style(Style::default().fg(Color::White).bg(Color::Black))
                .borders(Borders::ALL), 
            self.regions.chat
        );
        area.draw_widget(
            Block::default()
                .title(" Players ")
                .border_style(Style::default().fg(Color::White).bg(Color::Black))
                .borders(Borders::ALL), 
            self.regions.players
        );
        
        area.draw_stateful_child(&self.map_dragger, state);
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

    fn relayout(&mut self, relayout: WorldRelayout) {
        let space = relayout.absolute_layout_space.to_rect();
        let rects = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(space.width.saturating_sub(39)), Constraint::Min(39)].as_ref())
            .split(space);

        let right_pane_rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(rects[1].height.saturating_sub(5*4)), Constraint::Min(5*4)].as_ref())
            .split(rects[1]);
        let chat_event_rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(right_pane_rects[0]);

        self.regions.map = rects[0];
        self.regions.events = chat_event_rects[0];
        self.regions.chat = chat_event_rects[1];
        self.regions.players = right_pane_rects[1];

        let map_space = AbsoluteSpace::from_rect(
            Block::default()
                .borders(Borders::ALL)
                .inner(self.regions.map)
        );
        self.map_dragger.layout.set_size(UDim2::from_size2d(map_space.size));
        self.map_dragger.layout.set_position(UDim2::from_point2d(map_space.position));
        
        self.relayout_children(relayout);
    }
}