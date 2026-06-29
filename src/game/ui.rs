//! 游戏 UI 系统
//!
//! 负责游戏中的中文 HUD、植物卡片、操作提示与胜负结果画面。

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::game::assets::GameAssets;
use crate::game::catalog::ContentCatalog;
use crate::game::combat::Dead;
use crate::game::controls::{ControlBindings, key_label, mouse_label};
use crate::game::level::{LevelDefinition, LevelRuntime, PlantCards, SelectedPlant, SunBank};
use crate::game::plant::PlantKind;
use crate::game::projectile::ProjectileKind;
use crate::game::state::{GameState, LevelEntity};
use crate::game::theme::UiTheme;
use crate::game::zombie::Zombie;

/// 游戏 UI 插件，注册 HUD 初始化/更新、结果画面显示与清理系统。
pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_hud)
            .add_systems(
                Update,
                (select_plant_card_from_ui, update_hud)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
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

#[derive(SystemParam)]
struct HudParams<'w, 's> {
    bank: Res<'w, SunBank>,
    selected: Res<'w, SelectedPlant>,
    cards: Res<'w, PlantCards>,
    runtime: Res<'w, LevelRuntime>,
    definition: Res<'w, LevelDefinition>,
    catalog: Res<'w, ContentCatalog>,
    theme: Res<'w, UiTheme>,
    projectiles: Query<'w, 's, &'static ProjectileKind>,
    living_zombies: Query<'w, 's, (), (With<Zombie>, Without<Dead>)>,
    stats: Single<'w, 's, &'static mut Text, With<HudStatsText>>,
    labels: Query<
        'w,
        's,
        (
            &'static PlantCardLabel,
            &'static mut Text,
            &'static mut TextColor,
        ),
        Without<HudStatsText>,
    >,
    panels: Query<
        'w,
        's,
        (
            &'static PlantCardPanel,
            &'static mut BackgroundColor,
            &'static mut BorderColor,
        ),
    >,
}

/// 创建左侧状态/卡片区和右侧操作说明区。
fn setup_hud(
    mut commands: Commands,
    assets: Res<GameAssets>,
    theme: Res<UiTheme>,
    catalog: Res<ContentCatalog>,
    definition: Res<LevelDefinition>,
    controls: Res<ControlBindings>,
) {
    let font = assets.chinese_font.clone();

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
                    right: px(320),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(theme.panel_gap),
                    ..default()
                },
                Name::new("左侧状态区"),
            ))
            .with_children(|left| {
                left.spawn((
                    Node {
                        width: percent(100),
                        padding: UiRect::axes(px(16), px(9)),
                        border: UiRect::all(px(1)),
                        border_radius: BorderRadius::all(px(theme.panel_radius)),
                        ..default()
                    },
                    BackgroundColor(theme.hud_panel_background),
                    BorderColor::all(theme.hud_panel_border),
                    Name::new("关卡状态面板"),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("正在准备草坪……"),
                        TextFont {
                            font: font.clone(),
                            font_size: theme.hud_font_size,
                            ..default()
                        },
                        TextColor(theme.hud_text),
                        HudStatsText,
                    ));
                });

                left.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: px(theme.panel_gap),
                        ..default()
                    },
                    Name::new("植物卡片栏"),
                ))
                .with_children(|cards| {
                    for card_definition in &definition.cards {
                        let kind = card_definition.plant;
                        let plant = catalog.plant(kind);
                        cards
                            .spawn((
                                Button,
                                Node {
                                    width: px(theme.card_size.x),
                                    min_height: px(theme.card_size.y),
                                    padding: UiRect::axes(px(12), px(8)),
                                    border: UiRect::all(px(2)),
                                    border_radius: BorderRadius::all(px(theme.panel_radius)),
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(theme.card_background),
                                BorderColor::all(theme.card_border),
                                PlantCardPanel(kind),
                                Name::new(format!("{}卡片", plant.display_name)),
                            ))
                            .with_children(|card| {
                                card.spawn((
                                    Text::new(format!(
                                        "  {}\n      {} 太阳  ·  点击选择",
                                        plant.display_name, plant.price
                                    )),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: theme.card_font_size,
                                        ..default()
                                    },
                                    TextColor(theme.card_text),
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
                    border_radius: BorderRadius::all(px(theme.panel_radius)),
                    ..default()
                },
                BackgroundColor(theme.help_background),
                BorderColor::all(theme.help_border),
                Name::new("操作说明面板"),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(control_help(&controls)),
                    TextFont {
                        font: font.clone(),
                        font_size: theme.help_font_size,
                        ..default()
                    },
                    TextColor(theme.help_text),
                    TextLayout::new_with_justify(Justify::Left),
                ));
            });
        });
}

/// 点击植物卡片时切换当前选择。卡片即使暂时无法种植，也允许预先选择。
fn select_plant_card_from_ui(
    cards: Query<(&Interaction, &PlantCardPanel), Changed<Interaction>>,
    mut selected: ResMut<SelectedPlant>,
) {
    for (interaction, card) in &cards {
        if *interaction == Interaction::Pressed {
            selected.0 = card.0;
            break;
        }
    }
}

/// 刷新关卡统计和三张卡片的选中、余额及冷却视觉状态。
fn update_hud(mut params: HudParams) {
    let mut stats = format!(
        "太阳  {}     波次  {} / {}     已消灭  {}     时间  {:.1} 秒",
        params.bank.amount,
        params.runtime.waves_started(),
        params.definition.waves.len(),
        params.runtime.defeated_zombies,
        params.runtime.elapsed.as_secs_f32(),
    );
    let mut path_peas = 0;
    let mut physics_peas = 0;
    for kind in &params.projectiles {
        match kind {
            ProjectileKind::Pea => path_peas += 1,
            ProjectileKind::PhysicsPea => physics_peas += 1,
        }
    }
    stats.push_str(&format!(
        "\n场上  豌豆 {}（普通 {} / 物理 {}）    存活僵尸 {}",
        path_peas + physics_peas,
        path_peas,
        physics_peas,
        params.living_zombies.iter().count(),
    ));
    params.stats.0 = stats;

    // 卡片文字说明价格和当前不可用原因。
    for (label, mut text, mut color) in &mut params.labels {
        let kind = label.0;
        let plant = params.catalog.plant(kind);
        let remaining = params.cards.remaining(kind).as_secs_f32();
        let state = if remaining > 0.0 {
            format!("冷却 {remaining:.1} 秒")
        } else if params.bank.amount < plant.price {
            "太阳不足".to_string()
        } else {
            "可种植".to_string()
        };
        let marker = if params.selected.0 == kind {
            "▶"
        } else {
            " "
        };
        text.0 = format!(
            "{marker} {}\n      {} 太阳  ·  {state}",
            plant.display_name, plant.price,
        );
        color.0 = if params.selected.0 == kind {
            params.theme.card_selected_text
        } else if remaining > 0.0 || params.bank.amount < plant.price {
            params.theme.card_disabled_text
        } else {
            params.theme.card_text
        };
    }

    // 金色边框表示当前选中；不可用卡片降低亮度但仍保留说明文字。
    for (panel, mut background, mut border) in &mut params.panels {
        let kind = panel.0;
        let plant = params.catalog.plant(kind);
        let is_selected = params.selected.0 == kind;
        let unavailable = !params.cards.ready(kind) || params.bank.amount < plant.price;
        background.0 = if is_selected {
            params.theme.card_selected_background
        } else if unavailable {
            params.theme.card_disabled_background
        } else {
            params.theme.card_background
        };
        *border = BorderColor::all(if is_selected {
            params.theme.card_selected_border
        } else {
            params.theme.card_border
        });
    }
}

/// 显示中文胜利结果页。
fn show_victory(
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
fn show_defeat(
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

fn control_help(controls: &ControlBindings) -> String {
    let text = format!(
        "操作说明\n点击植物卡片  选择植物\n{}  放置植物 / 收集太阳\n{}  重新开始",
        mouse_label(controls.place_or_collect),
        key_label(controls.restart),
    );
    #[cfg(feature = "debug_tools")]
    {
        format!(
            "{text}\n{}  普通豌豆    {}  物理豌豆\n{}  显示碰撞体",
            key_label(controls.spawn_path_projectile),
            key_label(controls.spawn_physics_projectile),
            key_label(controls.toggle_physics),
        )
    }
    #[cfg(not(feature = "debug_tools"))]
    {
        text
    }
}

/// 离开结果状态时移除遮罩，确保重新开局不会残留旧界面。
fn cleanup_result(mut commands: Commands, results: Query<Entity, With<ResultEntity>>) {
    for entity in &results {
        commands.entity(entity).despawn();
    }
}
