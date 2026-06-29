//! Bevy PvZ 原型程序入口
//!
//! 基于 Bevy 引擎和 Rapier2D 物理引擎实现的 Plants vs. Zombies 单关原型。
//! 此处配置主窗口属性，加载 [`GamePlugin`] 并启动应用循环。

mod game;

use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};
use game::GamePlugin;
use game::physics::PhysicsDebugSettings;

/// 程序入口点。
///
/// 创建 Bevy 应用，配置 1200×720 的可调整大小窗口（标题 "Bevy PvZ - Rapier 2D prototype"），
/// 添加默认插件集和自定义 [`GamePlugin`]，然后启动运行循环。
fn main() {
    let options = match parse_cli(std::env::args().skip(1)) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}\n\n{}", cli_help());
            std::process::exit(2);
        }
    };
    if options.show_help {
        println!("{}", cli_help());
        return;
    }

    App::new()
        .insert_resource(PhysicsDebugSettings {
            enabled: options.debug_enabled,
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy PvZ - Rapier 2D prototype".into(),
                resolution: WindowResolution::new(1200, 720),
                present_mode: PresentMode::AutoVsync,
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GamePlugin)
        .run();
}

#[derive(Default)]
struct CliOptions {
    debug_enabled: bool,
    show_help: bool,
}

fn parse_cli(args: impl IntoIterator<Item = String>) -> Result<CliOptions, String> {
    let mut options = CliOptions::default();
    for argument in args {
        match argument.as_str() {
            "--debug" => options.debug_enabled = true,
            "--help" | "-h" => options.show_help = true,
            unknown => return Err(format!("未知参数: {unknown}")),
        }
    }
    Ok(options)
}

fn cli_help() -> &'static str {
    "用法: bevy-pvz [--debug]\n\n选项:\n  --debug    启动时显示 Rapier 碰撞体调试框\n  -h, --help 显示帮助"
}
