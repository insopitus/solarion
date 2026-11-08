//! Tabbed combat statistics panel in the top-left corner.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, ActivateOnPress, Button};

use crate::combat::components::{CombatRecord, Hero};
use crate::data::heroes::HeroRoster;

use super::theme::{
    format_number, gold, panel_bg, panel_soft_bg, stats_damage, stats_healing, stats_taken,
    tab_active, tab_hover, text_muted, text_primary, ui_text,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum StatsTabKind {
    #[default]
    Damage,
    Healing,
    Taken,
}

impl StatsTabKind {
    const ALL: [Self; 3] = [Self::Damage, Self::Healing, Self::Taken];

    fn label(self) -> &'static str {
        match self {
            Self::Damage => "输出",
            Self::Healing => "治疗",
            Self::Taken => "承伤",
        }
    }

    fn color(self) -> Color {
        match self {
            Self::Damage => stats_damage(),
            Self::Healing => stats_healing(),
            Self::Taken => stats_taken(),
        }
    }
}

#[derive(Resource, Default)]
struct ActiveStatsTab(StatsTabKind);

#[derive(Component)]
struct StatsTabButton(StatsTabKind);

#[derive(Component)]
struct StatsBody;

#[derive(Component)]
struct StatsEmpty;

/// One visual part of a statistics row. Using a single component type keeps
/// all row parts reachable through one query (and avoids conflicting `&mut
/// Node` queries).
#[derive(Component, Clone, Copy)]
enum StatsCell {
    Row(usize),
    Rank(usize),
    Name(usize),
    Value(usize),
    Fill(usize),
}

pub struct StatsPanelPlugin;

impl Plugin for StatsPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveStatsTab>()
            .add_systems(Startup, setup_stats_panels)
            .add_systems(Update, (update_tab_buttons, update_stats_body))
            .add_observer(switch_stats_tab);
    }
}

fn setup_stats_panels(mut commands: Commands, roster: Res<HeroRoster>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(12),
                top: px(12),
                width: px(310),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                padding: UiRect::all(px(9)),
                border_radius: BorderRadius::all(px(10)),
                ..default()
            },
            BackgroundColor(panel_bg()),
        ))
        .with_children(|panel| {
            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|row| {
                    row.spawn(ui_text("战斗统计", 14.0, text_muted()));
                    row.spawn(ui_text("点击标签切换", 10.0, text_muted()));
                });

            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(4),
                    height: px(26),
                    ..default()
                })
                .with_children(|tabs| {
                    for kind in StatsTabKind::ALL {
                        tabs.spawn((
                            StatsTabButton(kind),
                            Button,
                            ActivateOnPress,
                            Hovered::default(),
                            Node {
                                flex_grow: 1.0,
                                height: percent(100),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::all(px(5)),
                                ..default()
                            },
                            BackgroundColor(if kind == StatsTabKind::Damage {
                                tab_active()
                            } else {
                                panel_soft_bg()
                            }),
                        ))
                        .with_children(|button| {
                            button.spawn(ui_text(kind.label(), 12.0, text_primary()));
                        });
                    }
                });

            // Rows are pre-spawned for the roster size and driven by
            // `update_stats_body`; unused slots collapse with display: none.
            panel
                .spawn((
                    StatsBody,
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(4),
                        ..default()
                    },
                ))
                .with_children(|body| {
                    for slot in 0..roster.len() {
                        body.spawn((
                            StatsCell::Row(slot),
                            Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                height: px(18),
                                column_gap: px(6),
                                ..default()
                            },
                        ))
                        .with_children(|row| {
                            row.spawn((
                                StatsCell::Rank(slot),
                                ui_text("", 11.0, text_muted()),
                                Node {
                                    width: px(14),
                                    ..default()
                                },
                            ));
                            row.spawn((
                                StatsCell::Name(slot),
                                ui_text("", 11.0, text_primary()),
                                Node {
                                    width: px(56),
                                    ..default()
                                },
                            ));
                            row.spawn((
                                Node {
                                    flex_grow: 1.0,
                                    height: px(9),
                                    border_radius: BorderRadius::all(px(4)),
                                    overflow: Overflow::clip(),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
                            ))
                            .with_children(|track| {
                                track.spawn((
                                    StatsCell::Fill(slot),
                                    Node {
                                        width: percent(0),
                                        height: percent(100),
                                        border_radius: BorderRadius::all(px(4)),
                                        ..default()
                                    },
                                    BackgroundColor(stats_damage()),
                                ));
                            });
                            row.spawn((StatsCell::Value(slot), ui_text("", 11.0, gold())));
                        });
                    }

                    body.spawn((
                        StatsEmpty,
                        Visibility::Hidden,
                        ui_text("暂无数据", 12.0, text_muted()),
                    ));
                });
        });
}

fn switch_stats_tab(
    event: On<Activate>,
    buttons: Query<&StatsTabButton>,
    mut active: ResMut<ActiveStatsTab>,
) {
    if let Ok(button) = buttons.get(event.entity) {
        active.0 = button.0;
    }
}

fn update_tab_buttons(
    active: Res<ActiveStatsTab>,
    mut buttons: Query<(&StatsTabButton, &Hovered, &mut BackgroundColor)>,
) {
    for (button, hovered, mut background) in &mut buttons {
        let color = if button.0 == active.0 {
            tab_active()
        } else if hovered.get() {
            tab_hover()
        } else {
            panel_soft_bg()
        };
        if background.0 != color {
            *background = BackgroundColor(color);
        }
    }
}

fn update_stats_body(
    time: Res<Time>,
    roster: Res<HeroRoster>,
    active: Res<ActiveStatsTab>,
    heroes: Query<(&Hero, &CombatRecord)>,
    mut cells: Query<(
        &StatsCell,
        &mut Node,
        Option<&mut Text>,
        Option<&mut BackgroundColor>,
    )>,
    mut empty: Query<&mut Visibility, With<StatsEmpty>>,
    mut elapsed: Local<f32>,
) {
    *elapsed += time.delta_secs();
    if *elapsed < 0.2 {
        return;
    }
    *elapsed = 0.0;

    let mut rows: Vec<(&str, f32)> = match active.0 {
        StatsTabKind::Damage => heroes
            .iter()
            .map(|(hero, record)| (roster.hero(hero.index).name.as_str(), record.damage_dealt))
            .collect(),
        StatsTabKind::Healing => heroes
            .iter()
            .map(|(hero, record)| (roster.hero(hero.index).name.as_str(), record.healing_done))
            .collect(),
        StatsTabKind::Taken => heroes
            .iter()
            .map(|(hero, record)| (roster.hero(hero.index).name.as_str(), record.damage_taken))
            .collect(),
    };

    rows.retain(|(_, value)| *value > 0.5);
    rows.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let max = rows
        .first()
        .map(|(_, value)| *value)
        .unwrap_or(1.0)
        .max(1.0);
    let fill_color = active.0.color();

    for (cell, mut node, text, background) in &mut cells {
        match *cell {
            StatsCell::Row(slot) => {
                node.display = if slot < rows.len() {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            StatsCell::Rank(slot) => {
                if let (Some(mut text), Some(_)) = (text, rows.get(slot)) {
                    let new_text = format!("{}", slot + 1);
                    if text.0 != new_text {
                        text.0 = new_text;
                    }
                }
            }
            StatsCell::Name(slot) => {
                if let (Some(mut text), Some((name, _))) = (text, rows.get(slot)) {
                    if text.0 != *name {
                        text.0 = (*name).to_string();
                    }
                }
            }
            StatsCell::Value(slot) => {
                if let (Some(mut text), Some((_, value))) = (text, rows.get(slot)) {
                    let new_text = format_number(*value);
                    if text.0 != new_text {
                        text.0 = new_text;
                    }
                }
            }
            StatsCell::Fill(slot) => {
                let width = rows
                    .get(slot)
                    .map(|(_, value)| (*value / max * 100.0).clamp(0.0, 100.0))
                    .unwrap_or(0.0);
                node.width = percent(width);
                if let Some(mut background) = background
                    && background.0 != fill_color
                {
                    *background = BackgroundColor(fill_color);
                }
            }
        }
    }

    let is_empty = rows.is_empty();
    for mut visibility in &mut empty {
        let desired = if is_empty {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *visibility != desired {
            *visibility = desired;
        }
    }
}
