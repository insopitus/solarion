//! Floating combat text projected from world space into the UI layer.

use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::text::LineBreak;

use crate::ui::theme::text_font;

use super::components::{FloatingText, GameRoot, MainCamera};

const TEXT_WIDTH: f32 = 220.0;

pub fn spawn_floating_text(
    commands: &mut Commands,
    position: Vec3,
    screen_offset: Vec2,
    text: impl Into<String>,
    color: Color,
    font_size: f32,
) {
    let max_lifetime = 1.05;

    commands.spawn((
        GameRoot,
        FloatingText {
            world_position: position,
            screen_offset,
            lifetime: max_lifetime,
            max_lifetime,
            velocity: Vec3::new(0.0, 1.8, 0.0),
        },
        // Keep combat text above the HUD but below the skill tooltip.
        GlobalZIndex(80),
        Pickable::IGNORE,
        Text::new(text.into()),
        text_font(font_size),
        TextColor(color),
        TextLayout::new(Justify::Center, LineBreak::WordBoundary),
        TextShadow::default(),
        Node {
            position_type: PositionType::Absolute,
            width: px(TEXT_WIDTH),
            ..default()
        },
    ));
}

pub fn update_floating_texts(
    mut commands: Commands,
    time: Res<Time>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut query: Query<(Entity, &mut FloatingText, &mut Node, &mut TextColor)>,
) {
    let delta = time.delta_secs();
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };

    for (entity, mut floating, mut node, mut color) in &mut query {
        floating.lifetime -= delta;
        if floating.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        let velocity = floating.velocity;
        floating.world_position += velocity * delta;
        floating.velocity.y = (floating.velocity.y - 1.0 * delta).max(0.5);

        let world_position = floating.world_position;
        if let Ok(viewport) = camera.world_to_viewport(camera_transform, world_position) {
            node.left = px((viewport.x - TEXT_WIDTH * 0.5 + floating.screen_offset.x).max(0.0));
            node.top = px((viewport.y - 16.0 - floating.screen_offset.y).max(0.0));
        }

        // Hold full opacity briefly, then fade out over the remaining time.
        let alpha = (floating.lifetime / (floating.max_lifetime * 0.55)).clamp(0.0, 1.0);
        *color = TextColor(color.0.with_alpha(alpha));
    }
}
