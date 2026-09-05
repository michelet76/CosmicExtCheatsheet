// SPDX-License-Identifier: GPL-3.0-only

//! The cheatsheet overlay content.

use cosmic::iced::widget::responsive;
use cosmic::iced::{Alignment, Border, Color, Length, Shadow};
use cosmic::theme;
use cosmic::iced::widget::{Column, Row, Space};
use cosmic::widget::{button, container, icon, mouse_area, scrollable, text};
use cosmic::Element;

use crate::fl;
use crate::shortcuts::{Category, CheatsheetModel};

const COLUMN_WIDTH: f32 = 440.0;
const MAX_COLUMNS: usize = 4;

/// Build the full-surface overlay: a dimmed backdrop (click closes) with a
/// centred card listing every category.
pub fn overlay<'a, M>(model: &'a CheatsheetModel, on_close: M, on_settings: M, on_noop: M) -> Element<'a, M>
where
    M: Clone + 'static,
{
    let spacing = theme::spacing();

    let header = Row::new()
        .align_y(Alignment::Center)
        .spacing(spacing.space_s)
        .push(text::title2(fl!("app-title")))
        .push(text::caption(fl!("overlay-hint")).class(theme::Text::Default))
        .push(Space::new().width(Length::Fill))
        .push(
            button::icon(icon::from_name("preferences-system-symbolic"))
                .tooltip(fl!("overlay-settings"))
                .on_press(on_settings),
        )
        .push(
            button::icon(icon::from_name("window-close-symbolic"))
                .tooltip(fl!("overlay-close"))
                .on_press(on_close.clone()),
        );

    let body: Element<'a, M> = if model.is_empty() {
        container(text::body(fl!("overlay-empty")))
            .center(Length::Fill)
            .into()
    } else {
        responsive(move |size| {
            let columns = ((size.width / COLUMN_WIDTH).floor() as usize).clamp(1, MAX_COLUMNS);
            let mut cols: Vec<Vec<&'a Category>> = vec![Vec::new(); columns];
            let mut heights = vec![0usize; columns];
            // Greedy balance by row count.
            for category in &model.categories {
                let shortest = (0..columns).min_by_key(|i| heights[*i]).unwrap_or(0);
                heights[shortest] += category.entries.len() + 2;
                cols[shortest].push(category);
            }
            let mut layout = Row::new().spacing(spacing.space_m).align_y(Alignment::Start);
            for col in cols {
                let mut column = Column::new().spacing(spacing.space_m).width(Length::FillPortion(1));
                for category in col {
                    column = column.push(category_card(category));
                }
                layout = layout.push(column);
            }
            scrollable(container(layout).padding([0, spacing.space_xs, 0, 0]))
                .height(Length::Fill)
                .into()
        })
        .into()
    };

    let card = container(
        Column::new()
            .spacing(spacing.space_m)
            .push(header)
            .push(body),
    )
    .padding(spacing.space_l)
    .width(Length::Fill)
    .height(Length::Fill)
    .max_width(1800.0)
    .class(theme::Container::custom(|theme| {
        let cosmic = theme.cosmic();
        let mut bg = Color::from(cosmic.bg_color());
        bg.a = 0.98;
        container::Style {
            background: Some(bg.into()),
            border: Border {
                radius: cosmic.radius_l().into(),
                width: 1.0,
                color: Color::from(cosmic.bg_divider()),
            },
            shadow: Shadow::default(),
            text_color: Some(Color::from(cosmic.on_bg_color())),
            icon_color: Some(Color::from(cosmic.on_bg_color())),
            snap: false,
        }
    }));

    // The card swallows clicks; clicks on the backdrop close the overlay.
    let card = mouse_area(card).on_press(on_noop);

    let backdrop = container(card)
        .padding(theme::spacing().space_xl)
        .center(Length::Fill)
        .class(theme::Container::custom(|_| container::Style {
            background: Some(Color { a: 0.45, ..Color::BLACK }.into()),
            ..Default::default()
        }));

    mouse_area(backdrop).on_press(on_close).into()
}

fn category_card<'a, M: 'static>(category: &'a Category) -> Element<'a, M> {
    let spacing = theme::spacing();
    let mut list = Column::new().spacing(spacing.space_xxs);
    list = list.push(text::heading(&category.title));
    for entry in &category.entries {
        let mut combos = Column::new().spacing(spacing.space_xxxs).align_x(Alignment::End);
        for chips in &entry.bindings {
            combos = combos.push(chip_row(chips));
        }
        let line = Row::new()
            .align_y(Alignment::Center)
            .spacing(spacing.space_s)
            .push(text::body(&entry.label).width(Length::Fill))
            .push(combos);
        list = list.push(line);
    }
    container(list)
        .padding(spacing.space_s)
        .width(Length::Fill)
        .class(theme::Container::Card)
        .into()
}

fn chip_row<'a, M: 'static>(chips: &'a [String]) -> Element<'a, M> {
    let spacing = theme::spacing();
    let mut r = Row::new().spacing(spacing.space_xxxs).align_y(Alignment::Center);
    let last = chips.len().saturating_sub(1);
    for (i, chip) in chips.iter().enumerate() {
        r = r.push(chip_widget(chip));
        if i != last {
            r = r.push(text::caption("+"));
        }
    }
    r.into()
}

fn chip_widget<'a, M: 'static>(label: &'a str) -> Element<'a, M> {
    container(text::caption_heading(label))
        .padding([2, 7])
        .class(theme::Container::custom(|theme| {
            let cosmic = theme.cosmic();
            container::Style {
                background: Some(Color::from(cosmic.secondary_component_color()).into()),
                border: Border {
                    radius: cosmic.radius_s().into(),
                    width: 1.0,
                    color: Color::from(cosmic.secondary_container_divider()),
                },
                text_color: Some(Color::from(cosmic.on_secondary_component_color())),
                ..Default::default()
            }
        }))
        .into()
}
