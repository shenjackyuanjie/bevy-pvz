use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum GameSet {
    Spawn,
    LogicMovement,
    ContactRead,
    Combat,
    DeathAndCleanup,
    LevelOutcome,
}
