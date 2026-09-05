// SPDX-License-Identifier: GPL-3.0-only

//! The cheatsheet overlay content.

use cosmic::Element;
use cosmic::iced::widget::{Column, Row, Space, responsive};
use cosmic::iced::{Alignment, Border, Color, Length, Shadow};
use cosmic::theme;
use cosmic::widget::{button, container, icon, mouse_area, scrollable, search_input, text};

use crate::fl;
use crate::shortcuts::{CheatsheetModel, Entry, MatchedCategory};

const COLUMN_WIDTH: f32 = 440.0;
const MAX_COLUMNS: usize = 4;

/// Messages emitted by the overlay.
#[derive(Debug, Clone)]
pub enum Message {
    Close,
    OpenSettings,
    /// Run the command behind a clicked row.
    Run(String),
    SearchChanged(String),
    SearchClear,
    /// Enter in the search box.
    SearchSubmit,
    /// A click on the card itself; swallowed so it does not reach the backdrop.
    Noop,
}

/// Widget id of the search box, so the app can focus it when the overlay opens.
pub fn search_id() -> cosmic::widget::Id {
    cosmic::widget::Id::new("cheatsheet-search")
}

/// Build the full-surface overlay: a dimmed backdrop (click closes) with a
/// centred card listing every category that matches `query`.
pub fn overlay<'a>(model: &'a CheatsheetModel, query: &'a str) -> Element<'a, Message> {
    let spacing = theme::spacing();
    let search = model.search(query);
    let enter_target = search.enter_target().map(std::ptr::from_ref);

    let search_box = search_input(fl!("overlay-search"), query)
        .id(search_id())
        .on_input(Message::SearchChanged)
        .on_clear(Message::SearchClear)
        .on_submit(|_| Message::SearchSubmit)
        .width(Length::Fixed(320.0));

    let header = Row::new()
        .align_y(Alignment::Center)
        .spacing(spacing.space_s)
        .push(text::title2(fl!("app-title")))
        .push(text::caption(fl!("overlay-hint")).class(theme::Text::Default))
        .push(Space::new().width(Length::Fill))
        .push(search_box)
        .push(
            button::icon(icon::from_name("preferences-system-symbolic"))
                .tooltip(fl!("overlay-settings"))
                .on_press(Message::OpenSettings),
        )
        .push(
            button::icon(icon::from_name("window-close-symbolic"))
                .tooltip(fl!("overlay-close"))
                .on_press(Message::Close),
        );

    let body: Element<'a, Message> = if search.is_empty() {
        let message = if search.active { fl!("overlay-no-match") } else { fl!("overlay-empty") };
        container(text::body(message)).center(Length::Fill).into()
    } else {
        let matched = search.categories;
        responsive(move |size| {
            let columns = ((size.width / COLUMN_WIDTH).floor() as usize).clamp(1, MAX_COLUMNS);
            let mut cols: Vec<Vec<&MatchedCategory<'a>>> = vec![Vec::new(); columns];
            let mut heights = vec![0usize; columns];
            // Greedy balance by row count.
            for category in &matched {
                let shortest = (0..columns).min_by_key(|i| heights[*i]).unwrap_or(0);
                heights[shortest] += category.entries.len() + 2;
                cols[shortest].push(category);
            }
            let mut layout = Row::new().spacing(spacing.space_m).align_y(Alignment::Start);
            for col in cols {
                let mut column = Column::new().spacing(spacing.space_m).width(Length::FillPortion(1));
                for category in col {
                    column = column.push(category_card(category, enter_target));
                }
                layout = layout.push(column);
            }
            scrollable(container(layout).padding([0, spacing.space_xs, 0, 0]))
                .height(Length::Fill)
                .into()
        })
        .into()
    };

    let card = container(Column::new().spacing(spacing.space_m).push(header).push(body))
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
    let card = mouse_area(card).on_press(Message::Noop);

    let backdrop = container(card)
        .padding(spacing.space_xl)
        .center(Length::Fill)
        .class(theme::Container::custom(|_| container::Style {
            background: Some(Color { a: 0.45, ..Color::BLACK }.into()),
            ..Default::default()
        }));

    mouse_area(backdrop).on_press(Message::Close).into()
}

fn category_card<'a>(matched: &MatchedCategory<'a>, enter_target: Option<*const Entry>) -> Element<'a, Message> {
    let spacing = theme::spacing();
    let muted = muted_color();
    let mut list = Column::new().spacing(spacing.space_xxxs);
    list = list.push(text::heading(&matched.category.title));
    for entry in &matched.entries {
        let entry: &'a Entry = entry;
        let runnable = entry.command.is_some();
        let mut combos = Column::new().spacing(spacing.space_xxxs).align_x(Alignment::End);
        for chips in &entry.bindings {
            combos = combos.push(chip_row(chips, runnable));
        }
        let label = if runnable {
            text::body(&entry.label)
        } else {
            text::body(&entry.label).class(theme::Text::Color(muted))
        };
        let line = Row::new()
            .align_y(Alignment::Center)
            .spacing(spacing.space_s)
            .push(label.width(Length::Fill))
            .push(combos);
        let padding = [spacing.space_xxxs, spacing.space_xs];
        let line: Element<'a, Message> = match &entry.command {
            Some(command) => {
                let is_target = enter_target == Some(std::ptr::from_ref(entry));
                button::custom(line)
                    .class(if is_target { theme::Button::Suggested } else { theme::Button::Text })
                    .width(Length::Fill)
                    .padding(padding)
                    .on_press(Message::Run(command.clone()))
                    .into()
            }
            None => container(line).width(Length::Fill).padding(padding).into(),
        };
        list = list.push(line);
    }
    container(list)
        .padding(spacing.space_s)
        .width(Length::Fill)
        .class(theme::Container::Card)
        .into()
}

/// Text colour for rows that cannot be triggered by clicking.
fn muted_color() -> Color {
    let mut color = Color::from(theme::active().cosmic().on_bg_color());
    color.a = 0.5;
    color
}

fn chip_row<'a>(chips: &'a [String], enabled: bool) -> Element<'a, Message> {
    let spacing = theme::spacing();
    let mut r = Row::new().spacing(spacing.space_xxxs).align_y(Alignment::Center);
    let last = chips.len().saturating_sub(1);
    for (i, chip) in chips.iter().enumerate() {
        r = r.push(chip_widget(chip, enabled));
        if i != last {
            r = r.push(text::caption("+"));
        }
    }
    r.into()
}

fn chip_widget<'a>(label: &'a str, enabled: bool) -> Element<'a, Message> {
    let alpha = if enabled { 1.0 } else { 0.55 };
    container(text::caption_heading(label))
        .padding([2, 7])
        .class(theme::Container::custom(move |theme| {
            let cosmic = theme.cosmic();
            let with_alpha = |mut c: Color| {
                c.a *= alpha;
                c
            };
            container::Style {
                background: Some(with_alpha(Color::from(cosmic.secondary_component_color())).into()),
                border: Border {
                    radius: cosmic.radius_s().into(),
                    width: 1.0,
                    color: with_alpha(Color::from(cosmic.secondary_container_divider())),
                },
                text_color: Some(with_alpha(Color::from(cosmic.on_secondary_component_color()))),
                ..Default::default()
            }
        }))
        .into()
}
