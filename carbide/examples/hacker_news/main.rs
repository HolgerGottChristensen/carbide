mod hacker_news_api;
mod article;

use std::collections::HashSet;
use std::ops::Deref;
use chrono::{TimeZone, Utc};
use env_logger::Env;
use futures::future::join_all;
use carbide_controls::{capture, List, PlainButton, Selection};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::widget::*;
use carbide_wgpu::window::*;

use carbide_core::prelude::{EnvironmentFontSize, LocalState};
use carbide_core::state::{BoolState, ReadState, State, StateExt, StringState, TState, UsizeState};
use carbide_core::{lens, task};
use carbide_core::text::{FontFamily, FontWeight};
use carbide_core::widget::WidgetExt;
use reqwest::Response;
use carbide_core::color::TRANSPARENT;
use carbide_core::layout::BasicLayouter;
use crate::article::Article;


fn main() {
    env_logger::init();

    let mut window = Window::new(
        "Hacker-news example",
        1800,
        1000,
        None,
    );

    let mut family = FontFamily::new_from_paths("NotoSans", vec![
        "fonts/NotoSans/NotoSans-Regular.ttf",
        "fonts/NotoSans/NotoSans-Italic.ttf",
        "fonts/NotoSans/NotoSans-Bold.ttf",
    ]);

    window.add_font_family(family);

    let thumbs_up_icon = window.add_image_from_path("icons/thumb-up-line.png");
    let comments_icon = window.add_image_from_path("icons/chat-1-line.png");

    let env = window.environment_mut();

    let news_articles: TState<Option<Vec<Article>>> = LocalState::new(None);
    let selected_items: TState<HashSet<Id>> = LocalState::new(HashSet::new());

    let news_articles_for_index = news_articles.clone();

    let first_selected_article = selected_items.mapped(move |a: &HashSet<Id>| {
        match (news_articles_for_index.clone().value().deref(), a.iter().next()) {
            (Some(l), Some(id)) => {
                l.iter().find(|&a| &a.carbide_id == id).cloned()
            }
            _ => None
        }
    });

    fn id_function(article: &Article) -> Id { article.carbide_id }

    task!(env, news_articles := {
        let response: Vec<u64> = reqwest::get("https://hacker-news.firebaseio.com/v0/topstories.json").await.unwrap().json().await.unwrap();
        let texts = response.iter().take(25).map(|id| {
            async move {
                let mut article = reqwest::get(format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id)).await.unwrap().json::<Article>().await.unwrap();
                article.carbide_id = Id::new_v4();
                article
            }
        });

        Some(join_all(texts).await)
    });

    println!("Hello hacker news");

    let selected_items_delegate = selected_items.clone();

    let delegate = move |article: TState<Article>, index: UsizeState| -> Box<dyn Widget> {
        let selected_item = article.clone();

        let selected = selected_items_delegate.mapped(move |map: &HashSet<Id>| {
            map.contains(&id_function(&*selected_item.value()))
        });

        let top_padding = if *index.value() == 0 { 5.0 } else { 0.0 };

        let background_color = selected.mapped_env(|selected: &bool, _: &_, env: &Environment| {
            if *selected {
                env.env_color(EnvironmentColor::Accent)
            } else {
                TRANSPARENT
            }
        });

        VStack::new(vec![
            HStack::new(vec![
                Text::new(lens!(Article; article.title))
                    .font_weight(FontWeight::Bold),
                Spacer::new()
            ]),
            HStack::new(vec![

                Text::new(lens!(Article; |article| {
                    let dt = Utc.timestamp(article.time as i64, 0);
                    format!("by {} {}, {}", article.by, dt.format("%D"), dt.format("%I:%M %p"))
                })),

                Image::new_icon(thumbs_up_icon.clone())
                    .resizeable()
                    .frame(16, 16)
                    .accent_color(EnvironmentColor::SecondaryLabel),

                Text::new(lens!(Article; article.score))
                    .custom_flexibility(3),

                Image::new_icon(comments_icon.clone())
                    .resizeable()
                    .frame(16, 16)
                    .accent_color(EnvironmentColor::SecondaryLabel),

                Text::new(lens!(Article; article.descendants).unwrap_or_default())
                    .custom_flexibility(3),

            ]).spacing(3.0)
                .foreground_color(EnvironmentColor::SecondaryLabel),
        ]).spacing(0.0)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .padding(EdgeInsets::single(top_padding, 3.0, 10.0, 10.0))
            .background(Rectangle::new().fill(background_color).frame_fixed_width(6.0))
                .with_alignment(BasicLayouter::Leading)
    };

    let list = List::new(news_articles.unwrap_or_default(), delegate)
        .spacing(2.0)
        .selectable(id_function, selected_items);

    let loader = ZStack::new(vec![
        Rectangle::new()
            .fill(EnvironmentColor::SystemBackground),
        ProgressView::new()
    ]);

    window.set_widgets(
        HSplit::new(
            IfElse::new(news_articles.is_some().ignore_writes())
                .when_true(list)
                .when_false(loader),
            detail_view(first_selected_article)
        ).relative_to_start(400.0)
    );

    window.launch();
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