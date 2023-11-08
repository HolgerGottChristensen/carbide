use std::collections::HashSet;
use std::ops::Deref;

use chrono::{TimeZone, Utc};
use futures::stream::FuturesOrdered;
use futures::{FutureExt, StreamExt};
use reqwest::{Client, get};

use carbide::{a, lens, task, ui};
use carbide::{Application, Window};
use carbide::asynchronous::spawn;
use carbide::color::TRANSPARENT;
use carbide::draw::{Color, Dimension};
use carbide::environment::{Environment, EnvironmentColor, EnvironmentFontSize};
use carbide::layout::BasicLayouter;
use carbide::state::{AnyReadState, AnyState, IndexState, LocalState, Map1, Map2, ReadState, ReadStateExtNew, State, StateExt, TState};
use carbide::text::FontWeight;
use carbide::widget::*;
use carbide::widget::WidgetExt;
use carbide::controls::{List, PlainButton};

use crate::article::Article;
use crate::item::HNItem;

mod article;
mod item;

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let news_articles: LocalState<Option<Vec<Article>>> = LocalState::new(None);
    let selected_id: LocalState<Option<u64>> = LocalState::new(None);
    let current_hn_item: LocalState<Option<HNItem>> = LocalState::new(None);

    task!(news_articles := {
        let client = reqwest::Client::new();

        let response: Vec<u64> = client.get("https://hacker-news.firebaseio.com/v0/topstories.json").send().await.unwrap().json().await.unwrap();

        let mut futures = FuturesOrdered::new();
        response.iter().take(25).for_each(|id| {
            let client = client.clone();

            futures.push(
                async move {
                    client.get(format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id)).send().await.unwrap().json::<Article>().await.unwrap()
                }
            )
        });

        let articles = futures.collect::<Vec<_>>().await;

        //dbg!(&articles);
        Some(articles)
    });

    let selected_items_delegate = selected_id.clone();

    let current_hn_item_for_delegate = current_hn_item.clone();
    let delegate = move |article: Box<dyn AnyState<T=Article>>, index: Box<dyn AnyReadState<T=usize>>| -> Box<dyn AnyWidget> {
        let current_hn_item = current_hn_item_for_delegate.clone();
        let top_padding = if *index.value() == 0 { 5.0 } else { 0.0 };

        let selected = Map2::read_map(article.clone(), selected_items_delegate.clone(), |article, selected| {
            if let Some(id) = selected {
                article.id == *id
            } else {
                false
            }
        });

        let background_color = Map2::read_map(
            selected,
            EnvironmentColor::Accent.color(),
            |selected: &bool, base_color: &Color| {
                if *selected {
                    *base_color
                } else {
                    TRANSPARENT
                }
            },
        );

        let timestamp = Map1::read_map(article.clone(), |article| {
            let dt = Utc.timestamp(article.time as i64, 0);
            format!("by {} {}, {}", article.by, dt.format("%D"), dt.format("%I:%M %p"))
        });

        VStack::new((
            HStack::new((
                Text::new(lens!(article.title)).font_weight(FontWeight::Bold),
                Spacer::new(),
            )),
            HStack::new((
                Text::new(timestamp),
                Image::new_icon("icons/thumb-up-line.png")
                    .resizeable()
                    .frame(16.0, 16.0)
                    .accent_color(EnvironmentColor::SecondaryLabel),
                Text::new(lens!(article.score)).custom_flexibility(3),
                Image::new_icon("icons/chat-1-line.png")
                    .resizeable()
                    .frame(16.0, 16.0)
                    .accent_color(EnvironmentColor::SecondaryLabel),
                Text::new(Map1::read_map(article.clone(), |article| article.descendants.unwrap_or_default()))
                    .custom_flexibility(3),
            ))
            .spacing(3.0)
            .foreground_color(EnvironmentColor::SecondaryLabel),
        ))
        .spacing(0.0)
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .padding(EdgeInsets::single(top_padding, 3.0, 10.0, 10.0))
        .background(
            Rectangle::new()
                .fill(background_color)
                .frame_fixed_width(6.0),
        )
        .with_alignment(BasicLayouter::Leading)
            .on_click(a!(|_, _| {
                current_hn_item.clone().set_value(None);
                let id = article.value().id;

                task!(current_hn_item := {
                    let response = get(format!("https://hn.algolia.com/api/v1/items/{}", id)).await.unwrap();
                    let item = response.json::<HNItem>().await.unwrap();

                    println!("{:#?}", item);

                    Some(item)
                });
            }))
            .boxed()
    };

    let loader = ZStack::new((
        Rectangle::new().fill(EnvironmentColor::SystemBackground),
        ProgressView::new(),
    ));

    let widget = ui!(
        match news_articles {
            Some(news_articles) => {
                HSplit::new(
                    List::new(
                        news_articles.clone(),
                        delegate
                    ).selectable(selected_id.clone()),

                    match selected_id {
                        Some(f) => {
                            let index = Map2::read_map(news_articles.clone(), f.clone(), |a, i| {
                                a.iter().position(|r| r.id == *i).unwrap()
                            });
                            Box::new(detail_view(IndexState::new(news_articles, index), current_hn_item))
                        },
                        _ => Rectangle::new().fill(EnvironmentColor::SecondarySystemBackground).boxed(),
                    }
                ).relative_to_start(400.0).boxed()
            }
            _ => loader.boxed(),
        }
    );


    application.set_scene(Window::new(
        "Hacker-news example",
        Dimension::new(900.0, 500.0),
        widget,
    ).close_application_on_window_close());

    application.launch();
}

fn detail_view(selected: impl State<T=Article>, content: impl State<T=Option<HNItem>>) -> impl Widget {
    let selected_article_for_link = selected.clone();

    let title = lens!(selected.title);
    let url = lens!(selected.url);

    let link = ui!(match url {
        Some(url) => Text::new(url.clone()).on_click(a!(|_, _| {
            open::that(url.value().clone());
        })).background(Rectangle::new().fill(EnvironmentColor::Accent)).boxed(),
        _ => Text::new("No URL for the article").boxed()
    });

    let comments = ui!(match content {
        Some(content) => Scroll::new(Text::new(Map1::read_map(content, |c| format!("{:#?}", c))).boxed()).clip().boxed(),
        _ => Rectangle::new().fill(EnvironmentColor::Green).boxed()
    });

    ZStack::new((
        Rectangle::new().fill(EnvironmentColor::SecondarySystemBackground),
        VStack::new((
            HStack::new((
                Text::new(title).font_size(EnvironmentFontSize::Title),
                Spacer::new()
            )),
            link,
            comments,
            Spacer::new(),
        )).cross_axis_alignment(CrossAxisAlignment::Start)
            .spacing(0.0)
            .padding(EdgeInsets::single(0.0, 0.0, 10.0, 10.0)),
    ))
}
