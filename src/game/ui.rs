use bevy::prelude::*;

use super::level::{LevelDefinition, LevelRuntime, PlantCards, SelectedPlant, SunBank};
use super::plant::PlantKind;
use super::state::{GameState, LevelEntity};

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
struct HudText;

#[derive(Component)]
struct ResultEntity;

fn setup_hud(mut commands: Commands) {
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

fn update_hud(
    bank: Res<SunBank>,
    selected: Res<SelectedPlant>,
    cards: Res<PlantCards>,
    runtime: Res<LevelRuntime>,
    definition: Res<LevelDefinition>,
    mut text: Single<&mut Text, With<HudText>>,
) {
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

fn show_victory(mut commands: Commands) {
    show_result(&mut commands, "VICTORY", Color::srgb(0.35, 0.95, 0.35));
}

fn show_defeat(mut commands: Commands) {
    show_result(&mut commands, "DEFEAT", Color::srgb(1.0, 0.3, 0.25));
}

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

fn cleanup_result(mut commands: Commands, results: Query<Entity, With<ResultEntity>>) {
    for entity in &results {
        commands.entity(entity).despawn();
    }
}
