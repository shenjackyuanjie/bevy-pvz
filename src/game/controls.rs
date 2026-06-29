//! 正式玩法与可选调试功能共用的输入绑定。

use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct ControlBindings {
    pub place_or_collect: MouseButton,
    pub restart: KeyCode,
    #[cfg(feature = "debug_tools")]
    pub toggle_physics: KeyCode,
    #[cfg(feature = "debug_tools")]
    pub spawn_path_projectile: KeyCode,
    #[cfg(feature = "debug_tools")]
    pub spawn_physics_projectile: KeyCode,
}

impl Default for ControlBindings {
    fn default() -> Self {
        Self {
            place_or_collect: MouseButton::Left,
            restart: KeyCode::KeyR,
            #[cfg(feature = "debug_tools")]
            toggle_physics: KeyCode::KeyD,
            #[cfg(feature = "debug_tools")]
            spawn_path_projectile: KeyCode::KeyN,
            #[cfg(feature = "debug_tools")]
            spawn_physics_projectile: KeyCode::KeyP,
        }
    }
}

impl ControlBindings {
    pub fn validate(&self) -> Result<(), String> {
        #[cfg(not(feature = "debug_tools"))]
        let keys = [("restart", self.restart)];
        #[cfg(feature = "debug_tools")]
        let keys = [
            ("restart", self.restart),
            ("toggle_physics", self.toggle_physics),
            ("spawn_path_projectile", self.spawn_path_projectile),
            ("spawn_physics_projectile", self.spawn_physics_projectile),
        ];
        for (index, (name, key)) in keys.iter().enumerate() {
            if keys[..index].iter().any(|(_, previous)| previous == key) {
                return Err(format!(
                    "control key {key:?} is assigned more than once ({name})"
                ));
            }
        }
        Ok(())
    }
}

pub fn key_label(key: KeyCode) -> String {
    match key {
        KeyCode::Digit0 => "0".into(),
        KeyCode::Digit1 => "1".into(),
        KeyCode::Digit2 => "2".into(),
        KeyCode::Digit3 => "3".into(),
        KeyCode::Digit4 => "4".into(),
        KeyCode::Digit5 => "5".into(),
        KeyCode::Digit6 => "6".into(),
        KeyCode::Digit7 => "7".into(),
        KeyCode::Digit8 => "8".into(),
        KeyCode::Digit9 => "9".into(),
        other => format!("{other:?}").trim_start_matches("Key").to_string(),
    }
}

pub fn mouse_label(button: MouseButton) -> String {
    match button {
        MouseButton::Left => "鼠标左键".into(),
        MouseButton::Right => "鼠标右键".into(),
        MouseButton::Middle => "鼠标中键".into(),
        other => format!("{other:?}"),
    }
}
