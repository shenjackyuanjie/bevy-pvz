//! Bevy PvZ 原型程序入口
//!
//! 基于 Bevy 引擎和 Rapier2D 物理引擎实现的 Plants vs. Zombies 单关原型。
//! 此处配置主窗口属性，加载 [`GamePlugin`] 并启动应用循环。

mod game;

use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};
use game::GamePlugin;
use game::level::LevelDefinition;
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

    let mut app = App::new();
    app.insert_resource(PhysicsDebugSettings {
        enabled: options.debug_enabled,
    });
    if let Some(level_path) = &options.level_path {
        let level = LevelDefinition::load_from_file(level_path).unwrap_or_else(|error| {
            eprintln!("{error}");
            std::process::exit(2);
        });
        app.insert_resource(level);
    }
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
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
    level_path: Option<String>,
}

fn parse_cli(args: impl IntoIterator<Item = String>) -> Result<CliOptions, String> {
    let mut options = CliOptions::default();
    let mut args = args.into_iter();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--debug" => options.debug_enabled = true,
            "--level" | "-l" => {
                options.level_path = Some(
                    args.next()
                        .ok_or_else(|| format!("{argument} 需要关卡 RON 路径"))?,
                );
            }
            "--help" | "-h" => options.show_help = true,
            _ if argument.starts_with("--level=") => {
                options.level_path = Some(
                    argument
                        .strip_prefix("--level=")
                        .filter(|path| !path.is_empty())
                        .ok_or_else(|| "--level= 需要关卡 RON 路径".to_string())?
                        .to_string(),
                );
            }
            unknown => return Err(format!("未知参数: {unknown}")),
        }
    }
    Ok(options)
}

fn cli_help() -> &'static str {
    "用法: bevy-pvz [--debug] [--level <path>]\n\n选项:\n  --debug              启动时显示 Rapier 碰撞体调试框\n  -l, --level <path>   指定关卡 RON 文件\n  -h, --help           显示帮助"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_reads_level_path() {
        let options = parse_cli([
            "--debug".to_string(),
            "--level".to_string(),
            "assets/levels/level_row_three_physics_line.ron".to_string(),
        ])
        .unwrap();

        assert!(options.debug_enabled);
        assert_eq!(
            options.level_path.as_deref(),
            Some("assets/levels/level_row_three_physics_line.ron")
        );

        let options = parse_cli(["-l".to_string(), "custom.ron".to_string()]).unwrap();
        assert_eq!(options.level_path.as_deref(), Some("custom.ron"));

        let options = parse_cli(["--level=inline.ron".to_string()]).unwrap();
        assert_eq!(options.level_path.as_deref(), Some("inline.ron"));
    }

    #[test]
    fn cli_rejects_missing_level_path() {
        assert!(parse_cli(["--level".to_string()]).is_err());
        assert!(parse_cli(["--level=".to_string()]).is_err());
    }
}
