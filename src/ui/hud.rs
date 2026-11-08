//! Bottom party bar, wave banner and boss health bar.

use bevy::picking::Pickable;
use bevy::picking::events::PointerClick;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::LineBreak;

use crate::combat::components::{
    ActionState, Boss, ControlMode, Dead, Enemy, FinalStats, Health, Hero, Mana, StageProgress,
};
use crate::core::stats::MANA_PER_SEGMENT;
use crate::data::enemies::EnemyWaves;
use crate::data::heroes::{HeroDefinition, HeroRoster, Role};
use crate::progression::LevelProgress;

use super::cooldown::SkillCooldownMaterial;
use super::theme::{
    boss_bar_fill, format_number, gold, health_bar, health_bar_low, mana_empty, mana_filled,
    mana_partial, panel_bg, panel_soft_bg, skill_slot, text_font, text_muted, text_primary,
    ui_text,
};

#[derive(Component)]
enum HudText {
    Header(usize),
}

#[derive(Component)]
enum HudBar {
    Health(usize),
    Mana { hero: usize, segment: usize },
}

#[derive(Component)]
struct WaveBanner;

#[derive(Component)]
struct BossBarRoot;

#[derive(Component)]
struct BossBarFill;

#[derive(Component)]
struct BossBarText;

#[derive(Component, Clone, Copy)]
struct SkillSlot {
    hero: usize,
    skill: usize,
}

#[derive(Component, Clone, Copy)]
struct SkillCooldownOverlay {
    hero: usize,
    skill: usize,
}

#[derive(Component, Clone, Copy)]
struct SkillCooldownLabel {
    hero: usize,
    skill: usize,
}

#[derive(Component)]
struct SkillTooltip;

#[derive(Component)]
struct SkillTooltipText;

#[derive(Component, Clone, Copy)]
struct AutoToggle {
    hero: usize,
}

#[derive(Component, Clone, Copy)]
struct AutoToggleText {
    hero: usize,
}

#[derive(Clone, Copy)]
struct HeroSnapshot {
    level: u32,
    health: f32,
    max_health: f32,
    mana: f32,
    dead: bool,
}

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_hud).add_systems(
            Update,
            (
                update_hero_panels,
                update_wave_banner,
                update_boss_bar,
                update_skill_cooldowns,
                update_skill_tooltips,
                update_auto_toggles,
            ),
        );
    }
}

fn setup_hud(
    mut commands: Commands,
    roster: Res<HeroRoster>,
    asset_server: Res<AssetServer>,
    mut cooldown_materials: ResMut<Assets<SkillCooldownMaterial>>,
) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                bottom: px(0),
                height: px(196),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Stretch,
                column_gap: px(6),
                padding: UiRect::all(px(8)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.012, 0.018, 0.030, 0.92)),
        ))
        .with_children(|bar| {
            for (index, hero) in roster.heroes.iter().enumerate() {
                spawn_hero_card(bar, index, hero, &asset_server, &mut cooldown_materials);
            }
        });

    // Wave banner (top center).
    commands
        .spawn((Node {
            position_type: PositionType::Absolute,
            top: px(12),
            width: percent(100),
            justify_content: JustifyContent::Center,
            ..default()
        },))
        .with_children(|row| {
            row.spawn((WaveBanner, ui_text("", 20.0, gold())));
        });

    // Boss health bar, hidden until a boss is alive.
    commands
        .spawn((
            BossBarRoot,
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                top: px(50),
                width: percent(100),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|row| {
            row.spawn((
                Node {
                    width: px(480),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(3),
                    padding: UiRect::all(px(6)),
                    border_radius: BorderRadius::all(px(8)),
                    ..default()
                },
                BackgroundColor(panel_bg()),
            ))
            .with_children(|card| {
                card.spawn((BossBarText, ui_text("", 13.0, text_primary())));

                card.spawn((
                    Node {
                        width: percent(100),
                        height: px(14),
                        border_radius: BorderRadius::all(px(5)),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.60)),
                ))
                .with_children(|track| {
                    track.spawn((
                        BossBarFill,
                        Node {
                            width: percent(100),
                            height: percent(100),
                            border_radius: BorderRadius::all(px(5)),
                            ..default()
                        },
                        BackgroundColor(boss_bar_fill()),
                    ));
                });
            });
        });

    spawn_skill_tooltip(&mut commands);

    // Small control hint, away from the stats panel.
    commands
        .spawn((Node {
            position_type: PositionType::Absolute,
            top: px(14),
            right: px(12),
            ..default()
        },))
        .with_children(|row| {
            row.spawn(ui_text(
                "全自动战斗 · 结算界面按 R 重开",
                12.0,
                text_muted(),
            ));
        });
}

fn role_glyph(role: Role) -> &'static str {
    match role {
        Role::Tank => "坦",
        Role::Damage => "输",
        Role::Support => "辅",
        Role::Healer => "疗",
    }
}

fn spawn_hero_card(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    hero: &HeroDefinition,
    asset_server: &AssetServer,
    cooldown_materials: &mut Assets<SkillCooldownMaterial>,
) {
    parent
        .spawn((
            Node {
                flex_grow: 1.0,
                flex_basis: px(0),
                flex_direction: FlexDirection::Row,
                column_gap: px(8),
                padding: UiRect::all(px(6)),
                border_radius: BorderRadius::all(px(9)),
                ..default()
            },
            BackgroundColor(panel_soft_bg()),
        ))
        .with_children(|card| {
            // Stylised portrait stand-in until real character art exists.
            card.spawn((
                Node {
                    width: px(92),
                    height: percent(100),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: px(2),
                    border_radius: BorderRadius::all(px(8)),
                    ..default()
                },
                BackgroundColor(hero.color.color().with_alpha(0.20)),
            ))
            .with_children(|portrait| {
                portrait.spawn(ui_text(role_glyph(hero.role), 38.0, hero.color.color()));
                portrait.spawn(ui_text(hero.name.clone(), 12.0, text_primary()));

                portrait
                    .spawn((
                        AutoToggle { hero: index },
                        Hovered::default(),
                        Pickable::default(),
                        Node {
                            width: px(52),
                            height: px(18),
                            margin: UiRect::top(px(4)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(px(9)),
                            ..default()
                        },
                        BackgroundColor(health_bar()),
                    ))
                    .observe(
                        move |_click: On<PointerClick>,
                              mut heroes: Query<(&Hero, &mut ControlMode)>| {
                            for (hero, mut mode) in &mut heroes {
                                if hero.index == index {
                                    info!("AUTO toggle clicked hero {}", hero.index);
                                    *mode = match *mode {
                                        ControlMode::Auto => ControlMode::Manual,
                                        ControlMode::Manual => ControlMode::Auto,
                                    };
                                }
                            }
                        },
                    )
                    .with_children(|toggle| {
                        toggle.spawn((
                            AutoToggleText { hero: index },
                            ui_text("AUTO", 10.0, Color::BLACK),
                            Pickable::IGNORE,
                        ));
                    });
            });

            card.spawn(Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                row_gap: px(3),
                ..default()
            })
            .with_children(|info| {
                info.spawn((
                    HudText::Header(index),
                    ui_text(
                        format!("Lv.1  {} · {}", hero.name, hero.role.label()),
                        14.0,
                        hero.color.color(),
                    ),
                ));

                info.spawn((
                    Node {
                        width: percent(100),
                        height: px(12),
                        border_radius: BorderRadius::all(px(6)),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
                ))
                .with_children(|track| {
                    track.spawn((
                        HudBar::Health(index),
                        Node {
                            width: percent(100),
                            height: percent(100),
                            border_radius: BorderRadius::all(px(6)),
                            ..default()
                        },
                        BackgroundColor(health_bar()),
                    ));
                });

                // 5 mana segments; a partial segment keeps its own progress.
                info.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(3),
                    height: px(10),
                    ..default()
                })
                .with_children(|row| {
                    for segment in 0..5 {
                        row.spawn((
                            Node {
                                flex_grow: 1.0,
                                height: px(9),
                                border_radius: BorderRadius::all(px(2)),
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            BackgroundColor(mana_empty()),
                        ))
                        .with_children(|track| {
                            track.spawn((
                                HudBar::Mana {
                                    hero: index,
                                    segment,
                                },
                                Node {
                                    width: percent(0),
                                    height: percent(100),
                                    border_radius: BorderRadius::all(px(2)),
                                    ..default()
                                },
                                BackgroundColor(mana_partial()),
                            ));
                        });
                    }
                });

                // Three active skills; the icon fills each button.
                info.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(3),
                    height: px(70),
                    ..default()
                })
                .with_children(|row| {
                    for (skill_index, skill) in hero.skills.iter().enumerate() {
                        let cooldown = cooldown_materials.add(SkillCooldownMaterial {
                            progress: Vec4::ZERO,
                        });

                        row.spawn((
                            SkillSlot {
                                hero: index,
                                skill: skill_index,
                            },
                            Hovered::default(),
                            Node {
                                flex_grow: 1.0,
                                height: percent(100),
                                border_radius: BorderRadius::all(px(5)),
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            BackgroundColor(skill_slot()),
                        ))
                        .observe(
                            move |_click: On<PointerClick>,
                                  mut heroes: Query<(&Hero, &mut ActionState)>| {
                                for (hero, mut action) in &mut heroes {
                                    if hero.index == index {
                                        info!("skill click hero {} skill {}", hero.index, skill_index);
                                        action.pending_skill = Some(skill_index);
                                    }
                                }
                            },
                        )
                        .with_children(|cell| {
                            if !skill.icon.is_empty() {
                                let mut image =
                                    ImageNode::new(asset_server.load(skill.icon.clone()));
                                image.image_mode = NodeImageMode::Stretch;
                                cell.spawn((
                                    image,
                                    Node {
                                        width: percent(100),
                                        height: percent(100),
                                        ..default()
                                    },
                                    Pickable::IGNORE,
                                ));
                            }

                            // Radial cooldown wipe.
                            cell.spawn((
                                SkillCooldownOverlay {
                                    hero: index,
                                    skill: skill_index,
                                },
                                MaterialNode(cooldown),
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(0),
                                    right: px(0),
                                    top: px(0),
                                    bottom: px(0),
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ));

                            // Mana cost dots, floating over the icon.
                            let dots = (skill.mana_cost / 100.0).round().max(1.0) as usize;
                            cell.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    right: px(3),
                                    bottom: px(2),
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ))
                            .with_children(|overlay| {
                                overlay.spawn((
                                    ui_text("●".repeat(dots), 9.0, mana_filled()),
                                    Pickable::IGNORE,
                                ));
                            });

                            // Remaining cooldown in seconds, centered over the icon.
                            cell.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(0),
                                    right: px(0),
                                    top: px(0),
                                    bottom: px(0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ))
                            .with_children(|center| {
                                center.spawn((
                                    SkillCooldownLabel {
                                        hero: index,
                                        skill: skill_index,
                                    },
                                    Visibility::Hidden,
                                    ui_text("", 17.0, Color::WHITE),
                                    Pickable::IGNORE,
                                ));
                            });
                        });
                    }
                });
            });
        });
}

fn spawn_skill_tooltip(commands: &mut Commands) {
    commands
        .spawn((
            SkillTooltip,
            Visibility::Hidden,
            GlobalZIndex(200),
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                left: px(100),
                top: px(100),
                width: px(330),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(10)),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(8)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.015, 0.020, 0.035, 0.96)),
            BorderColor::all(gold().with_alpha(0.55)),
        ))
        .with_children(|tooltip| {
            tooltip.spawn((
                SkillTooltipText,
                Text::new(""),
                text_font(12.0),
                TextColor(text_primary()),
                TextLayout::new(Justify::Left, LineBreak::AnyCharacter),
                Node {
                    width: percent(100),
                    ..default()
                },
                Pickable::IGNORE,
            ));
        });
}

fn update_skill_cooldowns(
    roster: Res<HeroRoster>,
    heroes: Query<(&Hero, &ActionState, Option<&Dead>)>,
    mut materials: ResMut<Assets<SkillCooldownMaterial>>,
    overlays: Query<(&SkillCooldownOverlay, &MaterialNode<SkillCooldownMaterial>)>,
    mut labels: Query<(&SkillCooldownLabel, &mut Text, &mut Visibility)>,
) {
    let mut cooldowns: [Option<[f32; 3]>; 4] = [None; 4];
    for (hero, action, dead) in &heroes {
        if hero.index < cooldowns.len() {
            cooldowns[hero.index] = Some(if dead.is_some() {
                [0.0; 3]
            } else {
                action.cooldowns
            });
        }
    }

    for (slot, material_handle) in &overlays {
        let remaining = cooldowns
            .get(slot.hero)
            .copied()
            .flatten()
            .map(|cooldowns| cooldowns[slot.skill])
            .unwrap_or(0.0);
        let definition = roster.hero(slot.hero);
        let total = definition.skills[slot.skill].cooldown.max(0.001);
        let progress = (remaining / total).clamp(0.0, 1.0);

        if let Some(mut material) = materials.get_mut(&material_handle.0) {
            material.progress.x = progress;
        }
    }

    for (label, mut text, mut visibility) in &mut labels {
        let remaining = cooldowns
            .get(label.hero)
            .copied()
            .flatten()
            .map(|cooldowns| cooldowns[label.skill])
            .unwrap_or(0.0);

        if remaining > 0.05 {
            let new_text = if remaining >= 10.0 {
                format!("{remaining:.0}")
            } else {
                format!("{remaining:.1}")
            };
            if text.0 != new_text {
                text.0 = new_text;
            }
            if *visibility != Visibility::Inherited {
                *visibility = Visibility::Inherited;
            }
        } else if *visibility != Visibility::Hidden {
            *visibility = Visibility::Hidden;
        }
    }
}

fn update_skill_tooltips(
    windows: Query<&Window>,
    roster: Res<HeroRoster>,
    skills: Query<(&SkillSlot, &Hovered)>,
    mut tooltips: Query<(&mut Node, &mut Visibility), With<SkillTooltip>>,
    mut tooltip_texts: Query<&mut Text, With<SkillTooltipText>>,
    mut last_skill: Local<Option<(usize, usize)>>,
) {
    let Ok((mut node, mut visibility)) = tooltips.single_mut() else {
        return;
    };
    let Ok(mut tooltip_text) = tooltip_texts.single_mut() else {
        return;
    };

    let hovered = skills
        .iter()
        .find(|(_, hovered)| hovered.get())
        .map(|(slot, _)| (slot.hero, slot.skill));

    let Some((hero_index, skill_index)) = hovered else {
        if *visibility != Visibility::Hidden {
            *visibility = Visibility::Hidden;
        }
        *last_skill = None;
        return;
    };

    if *last_skill != Some((hero_index, skill_index)) {
        let skill = &roster.hero(hero_index).skills[skill_index];
        let dots = (skill.mana_cost / 100.0).round().max(1.0) as usize;
        let range_line = if skill.range > 0.0 {
            format!("\n射程：{:.1} 米", skill.range)
        } else {
            String::new()
        };
        let new_text = format!(
            "{}\n消耗：{} {} 法力 · 冷却：{:.1} 秒{}\n{}",
            skill.name,
            "●".repeat(dots),
            skill.mana_cost,
            skill.cooldown,
            range_line,
            skill.description
        );
        if tooltip_text.0 != new_text {
            tooltip_text.0 = new_text;
        }
        *last_skill = Some((hero_index, skill_index));
    }

    if *visibility != Visibility::Inherited {
        *visibility = Visibility::Inherited;
    }

    if let Ok(window) = windows.single()
        && let Some(cursor) = window.cursor_position()
    {
        let x = (cursor.x + 16.0).min(window.width() - 346.0).max(8.0);
        let y = (cursor.y + 18.0).min(window.height() - 160.0).max(8.0);

        let left = px(x);
        let top = px(y);
        if node.left != left {
            node.left = left;
        }
        if node.top != top {
            node.top = top;
        }
    }
}

fn update_auto_toggles(
    heroes: Query<(&Hero, &ControlMode)>,
    mut toggles: Query<(&AutoToggle, &mut BackgroundColor)>,
    mut texts: Query<(&AutoToggleText, &mut Text, &mut TextColor)>,
) {
    let auto_for = |index: usize| {
        heroes
            .iter()
            .find(|(hero, _)| hero.index == index)
            .map(|(_, mode)| *mode == ControlMode::Auto)
            .unwrap_or(true)
    };

    for (toggle, mut background) in &mut toggles {
        let color = if auto_for(toggle.hero) {
            health_bar()
        } else {
            Color::srgba(1.0, 1.0, 1.0, 0.10)
        };
        if background.0 != color {
            *background = BackgroundColor(color);
        }
    }

    for (label, mut text, mut color) in &mut texts {
        let auto = auto_for(label.hero);
        let new_text = if auto { "AUTO" } else { "手动" };
        if text.0 != new_text {
            text.0 = new_text.to_string();
        }
        let new_color = if auto { Color::BLACK } else { text_muted() };
        if color.0 != new_color {
            *color = TextColor(new_color);
        }
    }
}

fn update_hero_panels(
    roster: Res<HeroRoster>,
    heroes: Query<(
        &Hero,
        &Health,
        &FinalStats,
        &Mana,
        &LevelProgress,
        Option<&Dead>,
    )>,
    mut texts: Query<(&HudText, &mut Text, &mut TextColor)>,
    mut bars: Query<(&HudBar, &mut Node, &mut BackgroundColor)>,
) {
    let mut snapshots: [Option<HeroSnapshot>; 4] = [None; 4];
    for (hero, health, stats, mana, level, dead) in &heroes {
        if hero.index >= snapshots.len() {
            continue;
        }
        snapshots[hero.index] = Some(HeroSnapshot {
            level: level.level,
            health: health.current,
            max_health: stats.0.max_health,
            mana: mana.current,
            dead: dead.is_some(),
        });
    }

    for (text_kind, mut text, mut color) in &mut texts {
        match *text_kind {
            HudText::Header(index) => {
                let Some(snapshot) = snapshots.get(index).copied().flatten() else {
                    continue;
                };
                let definition = roster.hero(index);
                let suffix = if snapshot.dead { " · 阵亡" } else { "" };
                let new_text = format!(
                    "Lv.{}  {} · {} · {}{}",
                    snapshot.level,
                    definition.name,
                    definition.role.label(),
                    definition.attack_type.label(),
                    suffix
                );
                if text.0 != new_text {
                    text.0 = new_text;
                }
                let new_color = if snapshot.dead {
                    Color::srgb(0.5, 0.5, 0.55)
                } else {
                    definition.color.color()
                };
                if color.0 != new_color {
                    *color = TextColor(new_color);
                }
            }
        }
    }

    for (bar_kind, mut node, mut background) in &mut bars {
        match *bar_kind {
            HudBar::Health(index) => {
                let Some(snapshot) = snapshots.get(index).copied().flatten() else {
                    continue;
                };
                let ratio = if snapshot.max_health > 0.0 {
                    (snapshot.health / snapshot.max_health).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let width = percent(ratio * 100.0);
                if node.width != width {
                    node.width = width;
                }
                let new_color = if snapshot.dead {
                    Color::srgba(0.3, 0.3, 0.3, 0.6)
                } else if ratio <= 0.35 {
                    health_bar_low()
                } else {
                    health_bar()
                };
                if background.0 != new_color {
                    *background = BackgroundColor(new_color);
                }
            }
            HudBar::Mana { hero, segment } => {
                let Some(snapshot) = snapshots.get(hero).copied().flatten() else {
                    continue;
                };
                let segment_start = segment as f32 * MANA_PER_SEGMENT;
                let fill = ((snapshot.mana - segment_start) / MANA_PER_SEGMENT).clamp(0.0, 1.0);
                let width = percent(fill * 100.0);
                if node.width != width {
                    node.width = width;
                }
                // Full segments are bright; the in-progress segment is dim.
                let new_color = if fill >= 0.999 {
                    mana_filled()
                } else {
                    mana_partial()
                };
                if background.0 != new_color {
                    *background = BackgroundColor(new_color);
                }
            }
        }
    }
}

fn update_wave_banner(
    stage: Res<StageProgress>,
    waves: Res<EnemyWaves>,
    enemies: Query<(), (With<Enemy>, Without<Dead>)>,
    mut banners: Query<&mut Text, With<WaveBanner>>,
) {
    let Ok(mut text) = banners.single_mut() else {
        return;
    };

    let new_text = if stage.wave_index >= waves.len() {
        "关卡完成".to_string()
    } else {
        let wave_number = stage.wave_index + 1;
        let name = waves.name(stage.wave_index);
        let total = waves.len();
        if stage.active {
            format!(
                "第 {wave_number} / {total} 波 · {name} · 剩余敌人 {}",
                enemies.iter().count()
            )
        } else {
            format!("第 {wave_number} / {total} 波 · {name} · 推进中")
        }
    };

    if text.0 != new_text {
        text.0 = new_text;
    }
}

fn update_boss_bar(
    boss: Query<(&Name, &Health, &FinalStats, Option<&Dead>), With<Boss>>,
    mut roots: Query<&mut Visibility, With<BossBarRoot>>,
    mut fills: Query<&mut Node, With<BossBarFill>>,
    mut texts: Query<&mut Text, With<BossBarText>>,
) {
    let Ok(mut visibility) = roots.single_mut() else {
        return;
    };
    let Ok(mut fill) = fills.single_mut() else {
        return;
    };
    let Ok(mut text) = texts.single_mut() else {
        return;
    };

    let alive = boss
        .iter()
        .find(|(_, _, _, dead)| dead.is_none())
        .map(|(name, health, stats, _)| (name, health, stats));

    let Some((name, health, stats)) = alive else {
        *visibility = Visibility::Hidden;
        return;
    };

    if *visibility != Visibility::Inherited {
        *visibility = Visibility::Inherited;
    }

    let ratio = if stats.0.max_health > 0.0 {
        (health.current / stats.0.max_health).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let width = percent(ratio * 100.0);
    if fill.width != width {
        fill.width = width;
    }

    let new_text = format!(
        "{}   {} / {}",
        name,
        format_number(health.current),
        format_number(stats.0.max_health)
    );
    if text.0 != new_text {
        text.0 = new_text;
    }
}
