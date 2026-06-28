//! 共享游戏资源句柄。

use bevy::prelude::*;

pub const CHINESE_FONT_PATH: &str = "fonts/NotoSansSC-VF.ttf";

#[derive(Resource, Clone)]
pub struct GameAssets {
    pub chinese_font: Handle<Font>,
}

impl FromWorld for GameAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            chinese_font: asset_server.load(CHINESE_FONT_PATH),
        }
    }
}
