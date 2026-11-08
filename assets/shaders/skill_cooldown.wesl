import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct SkillCooldownMaterial {
    // x = remaining fraction (1.0 = just used, 0.0 = ready)
    progress: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> material: SkillCooldownMaterial;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let progress = clamp(material.progress.x, 0.0, 1.0);
    if (progress <= 0.001) {
        discard;
    }

    let centered = (in.uv - vec2<f32>(0.5)) * in.size;
    var angle = atan2(centered.x, -centered.y);
    if (angle < 0.0) {
        angle += 6.28318530718;
    }
    let normalized = angle / 6.28318530718;

    // The dark area shrinks exactly clockwise as the cooldown counts down.
    let boundary_angle = 1.0 - progress;
    if (normalized < boundary_angle) {
        discard;
    }

    let boundary = min(
        abs(normalized - boundary_angle),
        1.0 - abs(normalized - boundary_angle),
    );
    if (progress < 0.999 && boundary < 0.004) {
        return vec4<f32>(1.0, 0.78, 0.30, 0.75);
    }

    return vec4<f32>(0.18, 0.20, 0.28, 0.78);
}
