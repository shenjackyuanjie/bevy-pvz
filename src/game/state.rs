use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
    Victory,
    Defeat,
}

/// Everything owned by the current level carries this marker so restarts are reliable.
#[derive(Component, Debug)]
pub struct LevelEntity;
