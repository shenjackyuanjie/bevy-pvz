//! 胜负结果遮罩。

use bevy::prelude::*;

use super::ResultEntity;
use crate::game::assets::GameAssets;
use crate::game::controls::{ControlBindings, key_label};
use crate::game::theme::UiTheme;

/// 显示中文胜利结果页。
pub(super) fn show_victory(
    mut commands: Commands,
    assets: Res<GameAssets>,
    controls: Res<ControlBindings>,
    theme: Res<UiTheme>,
) {
    show_result(
        &mut commands,
        &assets,
        "胜利",
        &format!("草坪守住了！按 {} 再来一局", key_label(controls.restart)),
        theme.victory_text,
        &theme,
    );
}

/// 显示中文失败结果页。
pub(super) fn show_defeat(
    mut commands: Commands,
    assets: Res<GameAssets>,
    controls: Res<ControlBindings>,
    theme: Res<UiTheme>,
) {
    show_result(
        &mut commands,
        &assets,
        "失败",
        &format!(
            "僵尸突破了防线。按 {} 重新开始",
            key_label(controls.restart)
        ),
        theme.defeat_text,
        &theme,
    );
}

/// 构造覆盖整个窗口的结果遮罩，胜利和失败复用相同布局。
fn show_result(
    commands: &mut Commands,
    assets: &GameAssets,
    title: &str,
    subtitle: &str,
    color: Color,
    theme: &UiTheme,
) {
    let font = assets.chinese_font.clone();
    commands.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(18),
            ..default()
        },
        BackgroundColor(theme.result_background),
        ZIndex(100),
        ResultEntity,
        Name::new("游戏结果"),
        children![
            (
                Text::new(title),
                TextFont {
                    font: font.clone(),
                    font_size: theme.result_title_size,
                    ..default()
                },
                TextColor(color),
            ),
            (
                Text::new(subtitle),
                TextFont {
                    font,
                    font_size: theme.result_subtitle_size,
                    ..default()
                },
                TextColor(theme.result_subtitle),
            )
        ],
    ));
}

/// 离开结果状态时移除遮罩，确保重新开局不会残留旧界面。
pub(super) fn cleanup_result(mut commands: Commands, results: Query<Entity, With<ResultEntity>>) {
    for entity in &results {
        commands.entity(entity).despawn();
    }
}
