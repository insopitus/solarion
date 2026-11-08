//! Radial skill-cooldown overlay rendered with a custom UI material.

use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct SkillCooldownMaterial {
    /// `x` is the fraction of cooldown remaining (1 = just used, 0 = ready).
    #[uniform(0)]
    pub progress: Vec4,
}

impl UiMaterial for SkillCooldownMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/skill_cooldown.wesl".into()
    }
}

pub struct SkillCooldownPlugin;

impl Plugin for SkillCooldownPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<SkillCooldownMaterial>::default());
    }
}
