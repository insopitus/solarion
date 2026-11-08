//! Creates the persistent 3D scene and the run-scoped party/enemy entities.

use bevy::gltf::GltfAssetLabel;
use bevy::image::{
    ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};
use bevy::prelude::*;
use bevy::world_serialization::prelude::WorldAssetRoot;

use crate::core::rng::GameRng;
use crate::core::stats::{StatBlock, base_at_level};
use crate::data::enemies::EnemyDefinition;
use crate::data::equipment::{EquipmentConfig, generate_starter_loadout};
use crate::data::heroes::HeroRoster;
use crate::progression::LevelProgress;
use crate::ui::theme::text_font;

use super::components::{
    ActionState, ActiveBuffs, Boss, CombatProfile, CombatRecord, CombatTextAnchor, ControlMode,
    Enemy, EnemyAi, FinalStats, GameRoot, Health, Hero, HeroLoadout, HeroModel, MainCamera, Mana,
    PartyAnchor, StageProgress,
};

pub const PARTY_START_X: f32 = -12.0;
pub const WAVE_SPAWN_AHEAD: f32 = 8.0;
pub const PARTY_SPEED: f32 = 2.4;
pub const HERO_Y: f32 = 1.05;

/// The camera looks down at roughly 45 degrees from the side, so the view is
/// perpendicular to the +X travel direction while still separating characters
/// on the Z axis. Height equals distance to keep the pitch at 45 degrees.
pub const CAMERA_HEIGHT: f32 = 10.0;
pub const CAMERA_DISTANCE: f32 = 10.0;
pub const CAMERA_LOOK_Y: f32 = 1.2;
pub const CAMERA_VIEW_HEIGHT: f32 = 12.5;

/// Tiling arena floor. One texture repeat covers roughly 10 world units.
const FLOOR_TEXTURE: &str = "textures/floor/ruby-trim-castle-tile-001.png";
const FLOOR_REPEAT: Vec2 = Vec2::new(24.0, 9.0);

/// World-space point the camera should look at for a party at `anchor_x`.
pub fn camera_target(anchor_x: f32) -> Vec3 {
    Vec3::new(anchor_x + 2.0, CAMERA_LOOK_Y, 0.0)
}

/// Camera position for a party at `anchor_x`.
pub fn camera_position(anchor_x: f32) -> Vec3 {
    let target = camera_target(anchor_x);
    Vec3::new(
        target.x,
        target.y + CAMERA_HEIGHT,
        target.z + CAMERA_DISTANCE,
    )
}

pub fn wave_trigger_x(index: usize) -> f32 {
    -2.0 + 10.0 * index as f32
}

const HERO_OFFSETS: [Vec3; 4] = [
    Vec3::new(2.2, 0.0, 0.0),
    Vec3::new(0.6, 0.0, 1.7),
    Vec3::new(-1.0, 0.0, -1.7),
    Vec3::new(-2.6, 0.0, 0.0),
];

pub fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.68, 0.74, 0.92),
        brightness: 280.0,
        ..default()
    });

    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-6.0, 16.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Horizontal ground plane. The 45 degree camera sees it as a proper
    // surface, so characters standing at different Z values read as depth.
    let floor = asset_server
        .load_builder()
        .with_settings(|settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                mag_filter: ImageFilterMode::Linear,
                min_filter: ImageFilterMode::Linear,
                mipmap_filter: ImageFilterMode::Linear,
                anisotropy_clamp: 8,
                ..default()
            });
        })
        .load(FLOOR_TEXTURE);
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(240.0, 90.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.72, 0.74, 0.78),
            base_color_texture: Some(floor),
            uv_transform: bevy::math::Affine2::from_scale(FLOOR_REPEAT),
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(20.0, 0.0, -15.0),
    ));

    // Far backdrop wall: catches the eye and hides the horizon.
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(240.0, 6.0, 0.6))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.050, 0.064, 0.100),
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(20.0, 3.0, -8.5),
    ));

    // A handful of monoliths give the camera some parallax.
    let rock_mesh = meshes.add(Cuboid::new(1.4, 3.4, 1.4));
    let rock_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.17, 0.18, 0.24),
        perceptual_roughness: 0.95,
        ..default()
    });
    // Keep decorations behind the party but inside the 45 degree view.
    for (x, z, scale) in [
        (-4.0, -6.8, 1.0),
        (4.0, -7.3, 1.3),
        (12.0, -6.9, 0.8),
        (22.0, -7.2, 1.1),
        (30.0, -6.8, 1.4),
    ] {
        commands.spawn((
            Mesh3d(rock_mesh.clone()),
            MeshMaterial3d(rock_material.clone()),
            Transform::from_xyz(x, 1.7 * scale, z).with_scale(Vec3::splat(scale)),
        ));
    }

    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: CAMERA_VIEW_HEIGHT,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_translation(camera_position(PARTY_START_X))
            .looking_at(camera_target(PARTY_START_X), Vec3::Y),
    ));
}

pub fn cleanup_run(
    mut commands: Commands,
    run_entities: Query<Entity, (With<GameRoot>, Without<ChildOf>)>,
) {
    // Despawning the party anchor also despawns its hero children, so only
    // touch root entities to avoid issuing commands to already-despawned IDs.
    for entity in &run_entities {
        commands.entity(entity).despawn();
    }
}

/// Removes the previous run's entities, then rebuilds the party.
pub fn start_run(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    roster: Res<HeroRoster>,
    equipment: Res<EquipmentConfig>,
    mut rng: ResMut<GameRng>,
    mut stage: ResMut<StageProgress>,
    mut cameras: Query<&mut Transform, With<MainCamera>>,
) {
    *stage = StageProgress::default();

    // Keep the camera's established 45-degree orientation; only snap its
    // position back to the start of the run.
    if let Ok(mut camera) = cameras.single_mut() {
        camera.translation = camera_position(PARTY_START_X);
    }

    let anchor = commands
        .spawn((
            GameRoot,
            PartyAnchor,
            Transform::from_xyz(PARTY_START_X, 0.0, 0.0),
        ))
        .id();

    let hero_mesh = meshes.add(Capsule3d::new(0.45, 1.2));
    let ring_mesh = meshes.add(Cylinder::new(0.62, 0.03));
    info!("队伍已组建：{} 名角色", roster.len());

    for (index, definition) in roster.heroes.iter().enumerate() {
        let base = base_at_level(&definition.base, 1);
        let loadout = generate_starter_loadout(index, &mut rng, &equipment);
        let mut modifiers = definition.passive_modifiers.clone();
        modifiers.extend(loadout.modifiers());
        let stats = StatBlock::compute(&base, &modifiers);

        info!(
            "{} · {}（{}）",
            definition.name,
            definition.title,
            definition.role.label()
        );
        debug!(
            "  被动 {}：{}",
            definition.passive_name, definition.passive_description
        );
        for skill in &definition.skills {
            debug!(
                "  技能 {}（{} 法力 / {} 秒）：{}",
                skill.name, skill.mana_cost, skill.cooldown, skill.description
            );
        }
        for item in &loadout.items {
            debug!("  装备 {}", item.describe());
        }

        let offset = HERO_OFFSETS[index % HERO_OFFSETS.len()];
        let mut hero = commands.spawn((
            GameRoot,
            Hero { index },
            Name::new(definition.name.clone()),
            FinalStats(stats),
            Health::new(stats.max_health),
            Mana::default(),
            ActionState::default(),
            ControlMode::default(),
            ActiveBuffs::default(),
            CombatRecord::default(),
            CombatTextAnchor(2.55),
            HeroLoadout(loadout),
            LevelProgress::default(),
            Visibility::default(),
            Transform::from_xyz(offset.x, 0.0, offset.z),
        ));

        hero.with_children(|parent| {
            if definition.model.is_empty() {
                let material = materials.add(StandardMaterial {
                    base_color: definition.color.color(),
                    perceptual_roughness: 0.55,
                    metallic: 0.08,
                    ..default()
                });
                parent.spawn((
                    Mesh3d(hero_mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_xyz(0.0, HERO_Y, 0.0),
                ));
            } else {
                // The glTF faces +Z in its own space; turn it to face +X, the
                // direction the party advances and attacks.
                parent.spawn((
                    WorldAssetRoot(
                        asset_server
                            .load(GltfAssetLabel::Scene(0).from_asset(definition.model.clone())),
                    ),
                    HeroModel { hero_index: index },
                    Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2))
                        .with_scale(Vec3::splat(1.3)),
                ));
            }

            // A colored ring keeps each hero readable under a textured model.
            parent.spawn((
                Mesh3d(ring_mesh.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: definition.color.color().with_alpha(0.55),
                    alpha_mode: AlphaMode::Blend,
                    perceptual_roughness: 0.4,
                    ..default()
                })),
                Transform::from_xyz(0.0, 0.02, 0.0),
            ));

            parent.spawn((
                Text2d::new(definition.name.clone()),
                TextLayout::no_wrap(),
                text_font(32.0),
                TextColor(definition.color.color()),
                Transform::from_xyz(0.0, 2.15, 0.0).with_scale(Vec3::splat(0.030)),
            ));
        });

        let hero_entity = hero.id();
        commands.entity(anchor).add_child(hero_entity);
    }
}

pub fn spawn_wave(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    definitions: &[EnemyDefinition],
    party_x: f32,
) {
    let count: usize = definitions.iter().map(|enemy| enemy.count as usize).sum();
    let mut slot = 0usize;

    for definition in definitions {
        for _ in 0..definition.count {
            let z = (slot as f32 - (count as f32 - 1.0) * 0.5) * 3.2;
            let x = party_x + WAVE_SPAWN_AHEAD + (slot % 2) as f32 * 1.2;
            spawn_enemy(
                commands,
                meshes,
                materials,
                definition,
                Vec3::new(x, HERO_Y, z),
            );
            slot += 1;
        }
    }
}

fn spawn_enemy(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    definition: &EnemyDefinition,
    position: Vec3,
) {
    let stats = StatBlock::compute(&definition.base, &[]);
    let mut position = position;
    position.y = HERO_Y * definition.scale;
    let mesh = meshes.add(Capsule3d::new(0.5, 1.2));
    let material = materials.add(StandardMaterial {
        base_color: definition.color.color(),
        perceptual_roughness: 0.6,
        metallic: 0.05,
        ..default()
    });

    let mut entity = commands.spawn((
        GameRoot,
        Enemy,
        Name::new(definition.name.clone()),
        CombatProfile {
            level: definition.level,
            attack_type: definition.attack_type,
        },
        FinalStats(stats),
        Health::new(stats.max_health),
        CombatRecord::default(),
        CombatTextAnchor(1.3 * definition.scale),
        EnemyAi {
            attack_timer: 0.9,
            attack_interval: definition.attack_interval,
            attack_multiplier: definition.attack_multiplier,
            move_speed: definition.move_speed,
        },
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(position).with_scale(Vec3::splat(definition.scale)),
    ));

    if definition.is_boss {
        entity.insert(Boss);
        entity.with_children(|parent| {
            parent.spawn((
                Text2d::new("BOSS"),
                TextLayout::no_wrap(),
                text_font(42.0),
                TextColor(Color::srgb(1.0, 0.35, 0.30)),
                Transform::from_xyz(0.0, 1.6, 0.0).with_scale(Vec3::splat(0.018)),
            ));
        });
    }
}
