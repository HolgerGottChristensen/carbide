use carbide::draw::Dimension;
use carbide::environment::EnvironmentColor;
use carbide::state::{AnyReadState, AnyState, LocalState, Map1, ReadState, State};
use carbide::widget::{AnyWidget, ForEach, HStack, Rectangle, Text, VGrid, VGridColumn, VStack, Widget, WidgetExt};
use carbide::{lens, ui, Application, Window};

use crate::game_state::{BoardPosition, GameState, Player, Tile};

mod game_state;

fn main() {
    let mut application = Application::new();

    let game = LocalState::new(GameState::new(4));

    let rows = lens!(game.board);
    let current_player = lens!(game.current_player);

    let score = Map1::read_map(game.clone(), |g| g.score());

    let score_text = Map1::read_map(score, |s| format!("{:?}", s));

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
            VGrid::new(
                ForEach::new(rows, move |row, idx: Box<dyn AnyReadState<T=usize>>| {
                    let game_for_delegate = game_for_delegate.clone();

                    ForEach::new(row, move |item: Box<dyn AnyState<T=Tile>>, index: Box<dyn AnyReadState<T=usize>>| {
                        let idx = idx.clone();
                        let game_for_delegate = game_for_delegate.clone();

                        ui!(match item {
                            Tile::Empty => {
                                Rectangle::new().fill(EnvironmentColor::Gray)
                                    .on_click(move |_| {
                                        game_for_delegate.clone().value_mut().place(BoardPosition {x: *index.value(), y: *idx.value()});
                                    })
                            },
                            Tile::Filled(Player::Black) => Rectangle::new().fill(EnvironmentColor::Red),
                            Tile::Filled(Player::White) => Rectangle::new().fill(EnvironmentColor::Green),
                        })
                    })
                }),
                vec![
                    VGridColumn::Flexible { minimum: 0.0, maximum: f64::MAX },
                    VGridColumn::Flexible { minimum: 0.0, maximum: f64::MAX },
                    VGridColumn::Flexible { minimum: 0.0, maximum: f64::MAX },
                    VGridColumn::Flexible { minimum: 0.0, maximum: f64::MAX },
                    VGridColumn::Flexible { minimum: 0.0, maximum: f64::MAX },
                    VGridColumn::Flexible { minimum: 0.0, maximum: f64::MAX },
                    VGridColumn::Flexible { minimum: 0.0, maximum: f64::MAX },
                    VGridColumn::Flexible { minimum: 0.0, maximum: f64::MAX },
                ]
            )
        ))
            .padding(10.0)
    ));

    application.launch()

}
