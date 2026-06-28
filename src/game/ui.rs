//! 游戏 UI 系统
//!
//! 负责游戏中的用户界面元素：
//! - **HUD**：左上角显示太阳数量、波次进度、击杀数、时间以及三张植物卡片状态
//! - **操作提示**：右上角显示快捷键说明
//! - **结果画面**：胜利/失败时显示全屏遮罩和结果文字

use bevy::prelude::*;

use crate::game::level::{LevelDefinition, LevelRuntime, PlantCards, SelectedPlant, SunBank};
use crate::game::plant::PlantKind;
use crate::game::state::{GameState, LevelEntity};

/// 游戏 UI 插件，注册 HUD 初始化/更新、结果画面显示与清理系统。
pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_hud)
            .add_systems(Update, update_hud.run_if(in_state(GameState::Playing)))
            .add_systems(OnEnter(GameState::Victory), show_victory)
            .add_systems(OnEnter(GameState::Defeat), show_defeat)
            .add_systems(OnExit(GameState::Victory), cleanup_result)
            .add_systems(OnExit(GameState::Defeat), cleanup_result);
    }
}

/// 内部标记组件，标识 HUD 文字实体（方便单例查询更新）。
#[derive(Component)]
struct HudText;

/// 内部标记组件，标识结果画面实体（方便退出时清理）。
#[derive(Component)]
struct ResultEntity;

/// 初始化 HUD：创建左上角的游戏状态文字和右上角的操作提示。
fn setup_hud(mut commands: Commands) {
    // 左上角主 HUD 文字（太阳、波次、时间、卡片状态）
    commands.spawn((
        Text::new("Loading HUD..."),
        TextFont {
            font_size: 21.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(16),
            ..default()
        },
        HudText,
        LevelEntity,
        Name::new("Game HUD"),
    ));

    // 右上角操作提示
    commands.spawn((
        Text::new(
            "1/2/3 select plant | Left click plant/collect sun\n\
             N normal pea | P physics pea | D colliders | R restart",
        ),
        TextFont {
            font_size: 17.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.95, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            right: px(14),
            top: px(14),
            ..default()
        },
        LevelEntity,
        Name::new("Controls help"),
    ));
}

/// 每帧更新 HUD 文字内容，反映当前游戏状态。
///
/// 显示：太阳库存、当前波次/总波次、已消灭僵尸数、游戏用时，
/// 以及三张植物卡片的选中标记、名称、价格和冷却状态。
fn update_hud(
    bank: Res<SunBank>,
    selected: Res<SelectedPlant>,
    cards: Res<PlantCards>,
    runtime: Res<LevelRuntime>,
    definition: Res<LevelDefinition>,
    mut text: Single<&mut Text, With<HudText>>,
) {
    // 生成单张植物卡片的显示文字。
    let card = |number: usize, kind: PlantKind| {
        let remaining = cards.remaining(kind).as_secs_f32();
        let state = if remaining <= 0.0 {
            "ready".to_string()
        } else {
            format!("{remaining:.1}s")
        };
        let marker = if selected.0 == kind { ">" } else { " " };
        format!(
            "{marker}{number} {} [{} sun, {state}]",
            kind.display_name(),
            kind.price()
        )
    };

    text.0 = format!(
        "SUN: {}     WAVE: {}/{}     DEFEATED: {}     TIME: {:.1}s\n{}   {}   {}",
        bank.amount,
        runtime.next_spawn,
        definition.spawns.len(),
        runtime.defeated_zombies,
        runtime.elapsed.as_secs_f32(),
        card(1, PlantKind::Sunflower),
        card(2, PlantKind::Peashooter),
        card(3, PlantKind::WallNut),
    );
}

/// 显示胜利画面（绿色大字 VICTORY + 按 R 重新开始的提示）。
fn show_victory(mut commands: Commands) {
    show_result(&mut commands, "VICTORY", Color::srgb(0.35, 0.95, 0.35));
}

/// 显示失败画面（红色大字 DEFEAT + 按 R 重新开始的提示）。
fn show_defeat(mut commands: Commands) {
    show_result(&mut commands, "DEFEAT", Color::srgb(1.0, 0.3, 0.25));
}

/// 内部函数：创建全屏遮罩结果画面。
///
/// 包含一个半透明黑色背景、大号结果文字和重新开始的提示文字。
fn show_result(commands: &mut Commands, label: &str, color: Color) {
    commands.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(20),
            ..default()
        },
        BackgroundColor(Color::srgba(0.03, 0.04, 0.03, 0.92)),
        ResultEntity,
        children![
            (
                Text::new(label),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(color),
            ),
            (
                Text::new("Press R to play again"),
                TextFont {
                    font_size: 26.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            )
        ],
    ));
}

/// 退出结果画面时清理结果实体。
fn cleanup_result(mut commands: Commands, results: Query<Entity, With<ResultEntity>>) {
    for entity in &results {
        commands.entity(entity).despawn();
    }
}
