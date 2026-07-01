//! 常驻帧率覆盖层。

use std::time::Duration;

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::diagnostic::RenderDiagnosticsPlugin;

use crate::game::assets::GameAssets;
use crate::game::physics::PHYSICS_STEP_TIME;

const FPS_REFRESH_INTERVAL: Duration = Duration::from_millis(250);

pub(super) struct FpsOverlayPlugin;

impl Plugin for FpsOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            RenderDiagnosticsPlugin::default(),
        ))
        .add_systems(Startup, setup_fps_overlay)
        .add_systems(Update, update_fps_overlay);
    }
}

#[derive(Component)]
struct FpsText {
    refresh_timer: Timer,
}

fn setup_fps_overlay(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        Text::new("FPS --\n帧耗时 -- ms\n渲染 -- ms\n物理模拟 -- ms"),
        TextFont {
            font: assets.chinese_font.clone(),
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: px(12),
            bottom: px(10),
            padding: UiRect::axes(px(8), px(4)),
            border_radius: BorderRadius::all(px(5)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.62)),
        ZIndex(2000),
        FpsText {
            refresh_timer: Timer::new(FPS_REFRESH_INTERVAL, TimerMode::Repeating),
        },
        Name::new("FPS 显示"),
    ));
}

fn update_fps_overlay(
    time: Res<Time<Real>>,
    diagnostics: Res<DiagnosticsStore>,
    mut fps_text: Single<(&mut Text, &mut FpsText)>,
) {
    let (text, state) = &mut *fps_text;
    state.refresh_timer.tick(time.delta());
    if !state.refresh_timer.just_finished() {
        return;
    }

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|diagnostic| diagnostic.smoothed());
    let frame_time = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|diagnostic| diagnostic.smoothed());
    let physics_time = diagnostics
        .get(&PHYSICS_STEP_TIME)
        .and_then(|diagnostic| diagnostic.smoothed());

    let gpu_render_time = sum_render_diagnostics(&diagnostics, "elapsed_gpu");
    let (render_source, render_time) = if gpu_render_time.is_some() {
        ("GPU", gpu_render_time)
    } else {
        ("CPU", sum_render_diagnostics(&diagnostics, "elapsed_cpu"))
    };

    text.0 = format!(
        "FPS {}\n帧耗时 {} ms\n渲染 {render_source} {} ms\n物理模拟 {} ms",
        format_fps(fps),
        format_milliseconds(frame_time),
        format_milliseconds(render_time),
        format_milliseconds(physics_time),
    );
}

fn sum_render_diagnostics(diagnostics: &DiagnosticsStore, suffix: &str) -> Option<f64> {
    diagnostics
        .iter()
        .filter(|diagnostic| {
            let path = diagnostic.path().as_str();
            path.starts_with("render/") && path.ends_with(suffix)
        })
        .filter_map(|diagnostic| diagnostic.smoothed())
        .fold(None, |total, value| Some(total.unwrap_or(0.0) + value))
}

fn format_fps(value: Option<f64>) -> String {
    value.map_or_else(|| "--".to_string(), |value| format!("{value:.0}"))
}

fn format_milliseconds(value: Option<f64>) -> String {
    value.map_or_else(|| "--".to_string(), |value| format!("{value:.2}"))
}
