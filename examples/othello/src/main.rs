use carbide::draw::Dimension;
use carbide::environment::EnvironmentColor;
use carbide::state::{AnyReadState, AnyState, LocalState, Map1, ReadState, State};
use carbide::widget::{AnyWidget, ForEach, HStack, Rectangle, Text, LazyVGrid, GridItem, VStack, Widget, WidgetExt, Scroll};
use carbide::{lens, ui, Application, Window};

use crate::game_state::{BoardPosition, GameState, Player, Tile, TileInfo};

mod game_state;

fn main() {
    let mut application = Application::new();

    let game = LocalState::new(GameState::new(4));

    let rows = lens!(game.board);
    let current_player = lens!(game.current_player);

    let score = Map1::read_map(game.clone(), |g| g.score());

    let score_text = Map1::read_map(score, |s| format!("Red: {}, Green: {}", s.black, s.white));

    let player_indicator_color = Map1::read_map(current_player, |c| {
        match c {
            Player::White => EnvironmentColor::Green,
            Player::Black => EnvironmentColor::Red,
        }
    });

    let game_for_delegate = game.clone();

    application.set_scene(Window::new(
        "Othello game - Carbide",
        Dimension::new(600.0, 600.0),
        VStack::new((
            HStack::new((
                Rectangle::new().fill(player_indicator_color).frame(20.0, 20.0),
                Text::new(score_text),
            )),
            Scroll::new(LazyVGrid::new(
                vec![
                    GridItem::Flexible,
                    GridItem::Flexible,
                    GridItem::Flexible,
                    GridItem::Flexible,
                    GridItem::Flexible,
                    GridItem::Flexible,
                    GridItem::Flexible,
                    GridItem::Flexible,
                ],
                ForEach::new(rows, move |row, idx: Box<dyn AnyReadState<T=usize>>| {
                    let game_for_delegate = game_for_delegate.clone();

                    ForEach::new(row, move |item: Box<dyn AnyState<T=Tile>>, index: Box<dyn AnyReadState<T=usize>>| {
                        let idx = idx.clone();
                        let game_for_delegate = game_for_delegate.clone();

                        let info = lens!(item.info);
                        ui!(match info {
                            TileInfo::Empty => {
                                Rectangle::new().fill(EnvironmentColor::Gray)
                                    .on_click(move |_| {
                                        game_for_delegate.clone().value_mut().place(BoardPosition {x: *index.value(), y: *idx.value()});
                                    })
                            },
                            TileInfo::Filled(Player::Black) => Rectangle::new().fill(EnvironmentColor::Red),
                            TileInfo::Filled(Player::White) => Rectangle::new().fill(EnvironmentColor::Green),
                        }).aspect_ratio(Dimension::new(1.0, 1.0))
                    })
                })
            ).border())
        ))
            .padding(10.0)
    ));

    application.launch()

}
