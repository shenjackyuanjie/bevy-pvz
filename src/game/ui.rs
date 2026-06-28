//! 游戏 UI 系统
//!
//! 负责游戏中的中文 HUD、植物卡片、操作提示与胜负结果画面。

use bevy::prelude::*;

use crate::game::level::{LevelDefinition, LevelRuntime, PlantCards, SelectedPlant, SunBank};
use crate::game::plant::PlantKind;
use crate::game::state::{GameState, LevelEntity};

/// 所有界面与场景名牌共用的中文字体，避免 Bevy 默认字体缺少中文字形。
const UI_FONT: &str = "fonts/NotoSansSC-VF.ttf";

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

#[derive(Component)]
/// 标记顶部关卡数据文字，供每帧增量更新。
struct HudStatsText;

#[derive(Component)]
/// 标记植物卡片背景，并保存该卡片对应的植物类型。
struct PlantCardPanel(PlantKind);

#[derive(Component)]
/// 标记植物卡片文字，用于刷新价格、冷却与可用状态。
struct PlantCardLabel(PlantKind);

#[derive(Component)]
/// 标记胜负遮罩，离开结果状态时统一清理。
struct ResultEntity;

/// 创建左侧状态/卡片区和右侧操作说明区。
fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load(UI_FONT);

    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                position_type: PositionType::Absolute,
                ..default()
            },
            ZIndex(20),
            LevelEntity,
            Name::new("游戏界面"),
        ))
        .with_children(|root| {
            // 左侧集中显示即时数据与植物选择，半透明底板保证草坪上也能看清。
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: px(12),
                    left: px(16),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(8),
                    ..default()
                },
                Name::new("左侧状态区"),
            ))
            .with_children(|left| {
                left.spawn((
                    Node {
                        min_width: px(620),
                        padding: UiRect::axes(px(16), px(9)),
                        border: UiRect::all(px(1)),
                        border_radius: BorderRadius::all(px(10)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.035, 0.09, 0.045, 0.91)),
                    BorderColor::all(Color::srgba(0.68, 0.88, 0.43, 0.45)),
                    Name::new("关卡状态面板"),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("正在准备草坪……"),
                        TextFont {
                            font: font.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.96, 0.98, 0.88)),
                        HudStatsText,
                    ));
                });

                left.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: px(8),
                        ..default()
                    },
                    Name::new("植物卡片栏"),
                ))
                .with_children(|cards| {
                    for (key, kind) in [
                        (1, PlantKind::Sunflower),
                        (2, PlantKind::Peashooter),
                        (3, PlantKind::WallNut),
                    ] {
                        cards
                            .spawn((
                                Node {
                                    width: px(174),
                                    min_height: px(68),
                                    padding: UiRect::axes(px(12), px(8)),
                                    border: UiRect::all(px(2)),
                                    border_radius: BorderRadius::all(px(10)),
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.08, 0.14, 0.07, 0.92)),
                                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.16)),
                                PlantCardPanel(kind),
                                Name::new(format!("{}卡片", kind.display_name())),
                            ))
                            .with_children(|card| {
                                card.spawn((
                                    Text::new(format!(
                                        "  [{key}] {}\n      {} 太阳  ·  可种植",
                                        kind.display_name(),
                                        kind.price()
                                    )),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 15.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.88, 0.92, 0.80)),
                                    PlantCardLabel(kind),
                                ));
                            });
                    }
                });
            });

            // 操作说明独立靠右，避免与频繁变化的关卡数据混在一起。
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: px(12),
                    right: px(14),
                    width: px(290),
                    padding: UiRect::axes(px(15), px(11)),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(10)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.035, 0.055, 0.035, 0.88)),
                BorderColor::all(Color::srgba(0.75, 0.88, 0.62, 0.28)),
                Name::new("操作说明面板"),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(
                        "操作说明\n1 / 2 / 3  选择植物\n鼠标左键  放置植物 / 收集太阳\nN  普通豌豆    P  物理豌豆\nD  显示碰撞体  R  重新开始",
                    ),
                    TextFont {
                        font: font.clone(),
                        font_size: 15.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.88, 0.94, 0.83)),
                    TextLayout::new_with_justify(Justify::Left),
                ));
            });
        });
}

/// 刷新关卡统计和三张卡片的选中、余额及冷却视觉状态。
fn update_hud(
    bank: Res<SunBank>,
    selected: Res<SelectedPlant>,
    cards: Res<PlantCards>,
    runtime: Res<LevelRuntime>,
    definition: Res<LevelDefinition>,
    mut stats: Single<&mut Text, With<HudStatsText>>,
    mut labels: Query<(&PlantCardLabel, &mut Text, &mut TextColor)>,
    mut panels: Query<(&PlantCardPanel, &mut BackgroundColor, &mut BorderColor)>,
) {
    stats.0 = format!(
        "太阳  {}     波次  {} / {}     已消灭  {}     时间  {:.1} 秒",
        bank.amount,
        runtime.next_spawn,
        definition.spawns.len(),
        runtime.defeated_zombies,
        runtime.elapsed.as_secs_f32(),
    );

    // 卡片文字同时说明快捷键、价格和当前不可用原因。
    for (label, mut text, mut color) in &mut labels {
        let kind = label.0;
        let remaining = cards.remaining(kind).as_secs_f32();
        let state = if remaining > 0.0 {
            format!("冷却 {remaining:.1} 秒")
        } else if bank.amount < kind.price() {
            "太阳不足".to_string()
        } else {
            "可种植".to_string()
        };
        let marker = if selected.0 == kind { "▶" } else { " " };
        text.0 = format!(
            "{marker} [{}] {}\n      {} 太阳  ·  {state}",
            match kind {
                PlantKind::Sunflower => 1,
                PlantKind::Peashooter => 2,
                PlantKind::WallNut => 3,
            },
            kind.display_name(),
            kind.price(),
        );
        color.0 = if selected.0 == kind {
            Color::srgb(1.0, 0.95, 0.64)
        } else if remaining > 0.0 || bank.amount < kind.price() {
            Color::srgb(0.60, 0.64, 0.56)
        } else {
            Color::srgb(0.88, 0.92, 0.80)
        };
    }

    // 金色边框表示当前选中；不可用卡片降低亮度但仍保留说明文字。
    for (panel, mut background, mut border) in &mut panels {
        let kind = panel.0;
        let is_selected = selected.0 == kind;
        let unavailable = !cards.ready(kind) || bank.amount < kind.price();
        background.0 = if is_selected {
            Color::srgba(0.22, 0.30, 0.08, 0.96)
        } else if unavailable {
            Color::srgba(0.075, 0.085, 0.065, 0.90)
        } else {
            Color::srgba(0.08, 0.14, 0.07, 0.92)
        };
        *border = BorderColor::all(if is_selected {
            Color::srgb(1.0, 0.78, 0.24)
        } else {
            Color::srgba(1.0, 1.0, 1.0, 0.16)
        });
    }
}

/// 显示中文胜利结果页。
fn show_victory(mut commands: Commands, asset_server: Res<AssetServer>) {
    show_result(
        &mut commands,
        &asset_server,
        "胜利",
        "草坪守住了！按 R 再来一局",
        Color::srgb(0.48, 0.96, 0.36),
    );
}

/// 显示中文失败结果页。
fn show_defeat(mut commands: Commands, asset_server: Res<AssetServer>) {
    show_result(
        &mut commands,
        &asset_server,
        "失败",
        "僵尸突破了防线。按 R 重新开始",
        Color::srgb(1.0, 0.38, 0.28),
    );
}

/// 构造覆盖整个窗口的结果遮罩，胜利和失败复用相同布局。
fn show_result(
    commands: &mut Commands,
    asset_server: &AssetServer,
    title: &str,
    subtitle: &str,
    color: Color,
) {
    let font = asset_server.load(UI_FONT);
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
        BackgroundColor(Color::srgba(0.02, 0.035, 0.02, 0.93)),
        ZIndex(100),
        ResultEntity,
        Name::new("游戏结果"),
        children![
            (
                Text::new(title),
                TextFont {
                    font: font.clone(),
                    font_size: 76.0,
                    ..default()
                },
                TextColor(color),
            ),
            (
                Text::new(subtitle),
                TextFont {
                    font,
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.94, 0.96, 0.90)),
            )
        ],
    ));
}

/// 离开结果状态时移除遮罩，确保重新开局不会残留旧界面。
fn cleanup_result(mut commands: Commands, results: Query<Entity, With<ResultEntity>>) {
    for entity in &results {
        commands.entity(entity).despawn();
    }
}
