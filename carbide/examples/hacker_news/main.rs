mod article;
mod hacker_news_api;

use carbide_controls::{capture, List, PlainButton, Selection};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::widget::*;
use chrono::{TimeZone, Utc};
use env_logger::Env;
use futures::future::join_all;
use std::collections::HashSet;
use std::ops::Deref;
use futures::stream::FuturesOrdered;
use futures::StreamExt;

use crate::article::Article;
use carbide_core::color::TRANSPARENT;
use carbide_core::layout::BasicLayouter;
use carbide_core::prelude::{EnvironmentFontSize, LocalState};
use carbide_core::state::{
    BoolState, Map2, ReadState, State, StateExt, StringState, TState, UsizeState,
};
use carbide_core::text::{FontFamily, FontWeight};
use carbide_core::widget::WidgetExt;
use carbide_core::{lens, task, Color};
use reqwest::Response;
use carbide_core::draw::Dimension;
use carbide_wgpu::{Application, Window};

fn main() {
    env_logger::init();

    let mut application = Application::new();


    let mut family = FontFamily::new_from_paths(
        "NotoSans",
        vec![
            "fonts/NotoSans/NotoSans-Regular.ttf",
            "fonts/NotoSans/NotoSans-Italic.ttf",
            "fonts/NotoSans/NotoSans-Bold.ttf",
        ],
    );

    application.add_font_family(family);

    let env = application.environment_mut();

    let news_articles: TState<Option<Vec<Article>>> = LocalState::new(None);
    let selected_items: TState<HashSet<WidgetId>> = LocalState::new(HashSet::new());

    let news_articles_for_index = news_articles.clone();

    let first_selected_article = selected_items.mapped(move |a: &HashSet<WidgetId>| {
        match (
            news_articles_for_index.clone().value().deref(),
            a.iter().next(),
        ) {
            (Some(l), Some(id)) => l.iter().find(|&a| &a.carbide_id == id).cloned(),
            _ => None,
        }
    });

    fn id_function(article: &Article) -> WidgetId {
        article.carbide_id
    }

    task!(env, news_articles := {
        let client = reqwest::Client::new();

        let response: Vec<u64> = client.get("https://hacker-news.firebaseio.com/v0/topstories.json").send().await.unwrap().json().await.unwrap();

        let mut futures = FuturesOrdered::new();
        response.iter().take(25).for_each(|id| {
            let client = client.clone();

            futures.push(
                async move {
                    let mut article = client.get(format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id)).send().await.unwrap().json::<Article>().await.unwrap();
                    article.carbide_id = WidgetId::new();
                    article
                }
            )
        });

        Some(futures.collect::<Vec<_>>().await)
    });

    println!("Hello hacker news");

    let selected_items_delegate = selected_items.clone();

    let delegate = move |article: TState<Article>, index: UsizeState| -> Box<dyn Widget> {
        let selected_item = article.clone();

        let selected = selected_items_delegate.mapped(move |map: &HashSet<WidgetId>| {
            map.contains(&id_function(&*selected_item.value()))
        });

        let top_padding = if *index.value() == 0 { 5.0 } else { 0.0 };

        let background_color = Map2::read_map(
            selected.clone(),
            EnvironmentColor::Accent.state(),
            |selected: &bool, base_color: &Color| {
                if *selected {
                    *base_color
                } else {
                    TRANSPARENT
                }
            },
        )
        .ignore_writes();

        VStack::new(vec![
            HStack::new(vec![
                Text::new(lens!(Article; article.title)).font_weight(FontWeight::Bold),
                Spacer::new(),
            ]),
            HStack::new(vec![
                Text::new(lens!(Article; |article| {
                    let dt = Utc.timestamp(article.time as i64, 0);
                    format!("by {} {}, {}", article.by, dt.format("%D"), dt.format("%I:%M %p"))
                })),
                Image::new_icon(Application::assets().join("icons/thumb-up-line.png"))
                    .resizeable()
                    .frame(16, 16)
                    .accent_color(EnvironmentColor::SecondaryLabel),
                Text::new(lens!(Article; article.score)).custom_flexibility(3),
                Image::new_icon(Application::assets().join("icons/chat-1-line.png"))
                    .resizeable()
                    .frame(16, 16)
                    .accent_color(EnvironmentColor::SecondaryLabel),
                Text::new(lens!(Article; article.descendants).unwrap_or_default())
                    .custom_flexibility(3),
            ])
            .spacing(3.0)
            .foreground_color(EnvironmentColor::SecondaryLabel),
        ])
        .spacing(0.0)
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .padding(EdgeInsets::single(top_padding, 3.0, 10.0, 10.0))
        .background(
            Rectangle::new()
                .fill(background_color)
                .frame_fixed_width(6.0),
        )
        .with_alignment(BasicLayouter::Leading)
    };

    let list = List::new(news_articles.unwrap_or_default(), delegate)
        .spacing(2.0)
        .selectable(id_function, selected_items);

    let loader = ZStack::new(vec![
        Rectangle::new().fill(EnvironmentColor::SystemBackground),
        ProgressView::new(),
    ]);

    application.set_scene(Window::new(
        "Hacker-news example",
        Dimension::new(900.0, 500.0),
        HSplit::new(
            IfElse::new(news_articles.is_some().ignore_writes())
                .when_true(list)
                .when_false(loader),
            detail_view(first_selected_article),
        )
            .relative_to_start(400.0),
    ).close_application_on_window_close());

    application.launch();
}

fn detail_view(selected_article: TState<Option<Article>>) -> Box<dyn Widget> {
    let selected_article_for_link = selected_article.clone();
    let link =
        PlainButton::new(
            Text::new(lens!(Option<Article>; |selected_article| {
                selected_article.as_ref().and_then(|a| a.url.clone()).unwrap_or("No url to show".to_string())
            })).foreground_color(EnvironmentColor::SecondaryLabel)
        ).on_click(move |_: &mut Environment, _:_| {
            let selected_article = selected_article_for_link.clone();
            println!("Clicked");
            selected_article.value().as_ref().and_then(|article| article.url.as_ref()).map(|a| open::that(a));
        });

    ZStack::new(vec![
        Rectangle::new()
            .fill(EnvironmentColor::SecondarySystemBackground),
        VStack::new(vec![
            HStack::new(vec![
                Text::new(lens!(Option<Article>; |selected_article| {
                    selected_article.as_ref().map(|a| a.title.clone()).unwrap_or("No selected articles".to_string())
                })).font_size(EnvironmentFontSize::Title),
                Spacer::new()
            ]),
            link,
            Spacer::new(),
        ]).cross_axis_alignment(CrossAxisAlignment::Start)
            .spacing(0.0)
            .padding(EdgeInsets::single(0.0, 0.0, 10.0, 10.0)),
    ])
}
