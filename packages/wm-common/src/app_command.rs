use std::{iter, path::PathBuf};

use clap::{error::KindFormatter, Args, Parser, ValueEnum};
use serde::{Deserialize, Deserializer, Serialize};
use tracing::Level;
use uuid::Uuid;
use wm_platform::{Delta, Direction, LengthValue, OpacityValue};

use crate::TilingDirection;

const VERSION: &str = env!("VERSION_NUMBER");

#[derive(Clone, Debug, Parser)]
#[clap(name = "glazewm", author, version = VERSION, about, long_about = None)]
pub enum AppCommand {
  /// 启动窗口管理器。
  Start {
    /// 用户配置文件的自定义路径。
    ///
    /// 默认路径为 %userprofile%/.glzr/glazewm/config.yaml
    #[clap(short = 'c', long = "config", value_hint = clap::ValueHint::FilePath)]
    config_path: Option<PathBuf>,

    #[clap(flatten)]
    verbosity: Verbosity,
  },

  /// 获取并输出窗口管理器的指定状态。
  ///
  /// 需要窗口管理器已在运行。
  #[clap(alias = "q")]
  Query {
    #[clap(subcommand)]
    command: QueryCommand,
  },

  /// 执行窗口管理命令。
  ///
  /// 需要窗口管理器已在运行。
  #[clap(alias = "c")]
  Command {
    #[clap(long = "id")]
    subject_container_id: Option<Uuid>,

    #[clap(subcommand)]
    command: InvokeCommand,
  },

  /// 订阅一个或多个窗口管理器事件（例如 window_close），
  /// 并持续输出收到的事件。
  ///
  /// 需要窗口管理器已在运行。
  Sub {
    /// 要订阅的窗口管理器事件。
    #[clap(short = 'e', long, value_enum, num_args = 1..)]
    events: Vec<SubscribableEvent>,
  },

  /// 取消现有的事件订阅。
  ///
  /// 需要窗口管理器已在运行。
  Unsub {
    /// 要取消的订阅 ID。
    #[clap(long = "id")]
    subscription_id: Uuid,
  },
}

impl AppCommand {
  /// 从命令行参数解析 AppCommand。
  ///
  /// 未提供参数时默认为 AppCommand::Start。
  #[must_use]
  pub fn parse_with_default(args: &Vec<String>) -> Self {
    if args.len() == 1 {
      AppCommand::Start {
        config_path: None,
        verbosity: Verbosity {
          verbose: false,
          quiet: false,
        },
      }
    } else {
      AppCommand::parse_from(args)
    }
  }
}

/// Verbosity flags to be used with `#[command(flatten)]`.
#[derive(Args, Clone, Debug)]
#[clap(about = None, long_about = None)]
pub struct Verbosity {
  /// 启用详细日志。
  #[clap(short = 'v', long, action)]
  verbose: bool,

  /// 禁用日志。
  #[clap(short = 'q', long, action, conflicts_with = "verbose")]
  quiet: bool,
}

impl Verbosity {
  /// 根据详细程度选项获取日志级别。
  #[must_use]
  pub fn level(&self) -> Level {
    match (self.verbose, self.quiet) {
      (true, _) => Level::DEBUG,
      (_, true) => Level::ERROR,
      _ => Level::INFO,
    }
  }
}

#[derive(Clone, Debug, Parser)]
pub enum QueryCommand {
  /// 输出应用程序的元数据（例如版本号）。
  AppMetadata,
  /// 输出当前启用的快捷键模式。
  BindingModes,
  /// 输出当前获得焦点的容器（窗口或
  /// 空工作区）。
  Focused,
  /// 输出当前获得焦点的容器的平铺方向。
  TilingDirection,
  /// 输出所有显示器。
  Monitors,
  /// 输出所有窗口。
  Windows,
  /// 输出所有活动工作区。
  Workspaces,
  /// 输出窗口管理器是否已暂停。
  Paused,
}

#[derive(Clone, Debug, PartialEq, ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum SubscribableEvent {
  All,
  ApplicationExiting,
  BindingModesChanged,
  FocusChanged,
  FocusedContainerMoved,
  MonitorAdded,
  MonitorUpdated,
  MonitorRemoved,
  TilingDirectionChanged,
  UserConfigChanged,
  WindowManaged,
  WindowUnmanaged,
  WorkspaceActivated,
  WorkspaceDeactivated,
  WorkspaceUpdated,
  PauseChanged,
}

#[derive(Clone, Debug, Parser, PartialEq, Serialize)]
pub enum InvokeCommand {
  AdjustBorders(InvokeAdjustBordersCommand),
  Close,
  Focus(InvokeFocusCommand),
  Ignore,
  Move(InvokeMoveCommand),
  MoveWorkspace {
    #[clap(long)]
    direction: Direction,
  },
  Position(InvokePositionCommand),
  Resize(InvokeResizeCommand),
  UpdateWorkspaceConfig {
    #[clap(long, allow_hyphen_values = true)]
    workspace: Option<String>,
    #[clap(flatten)]
    new_config: InvokeUpdateWorkspaceConfig,
  },
  SetFloating {
    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    shown_on_top: Option<bool>,

    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    centered: Option<bool>,

    #[clap(long, allow_hyphen_values = true)]
    x_pos: Option<i32>,

    #[clap(long, allow_hyphen_values = true)]
    y_pos: Option<i32>,

    #[clap(long, allow_hyphen_values = true)]
    width: Option<LengthValue>,

    #[clap(long, allow_hyphen_values = true)]
    height: Option<LengthValue>,
  },
  SetFullscreen {
    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    shown_on_top: Option<bool>,

    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    maximized: Option<bool>,
  },
  SetMinimized,
  SetTiling,
  SetTitleBarVisibility {
    #[clap(required = true, value_enum)]
    visibility: TitleBarVisibility,
  },
  SetTransparency(SetTransparencyCommand),
  ShellExec {
    #[clap(long, action)]
    hide_window: bool,

    #[clap(required = true, trailing_var_arg = true)]
    command: Vec<String>,
  },
  // Reuse `InvokeResizeCommand` struct.
  Size(InvokeResizeCommand),
  ToggleFloating {
    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    shown_on_top: Option<bool>,

    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    centered: Option<bool>,
  },
  ToggleFullscreen {
    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    shown_on_top: Option<bool>,

    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    maximized: Option<bool>,
  },
  ToggleMinimized,
  ToggleTiling,
  ToggleTilingDirection,
  SetTilingDirection {
    #[clap(required = true)]
    tiling_direction: TilingDirection,
  },
  WmCycleFocus {
    #[clap(long, default_value_t = false)]
    omit_floating: bool,

    #[clap(long, default_value_t = false)]
    omit_fullscreen: bool,

    #[clap(long, default_value_t = true)]
    omit_minimized: bool,

    #[clap(long, default_value_t = false)]
    omit_tiling: bool,
  },
  WmDisableBindingMode {
    #[clap(long)]
    name: String,
  },
  WmEnableBindingMode {
    #[clap(long)]
    name: String,
  },
  WmExit,
  WmRedraw,
  WmReloadConfig,
  WmTogglePause,
}

impl<'de> Deserialize<'de> for InvokeCommand {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    // Clap expects an array of string slices where the first argument is
    // the binary name/path. When deserializing commands from the user
    // config, we therefore have to prepend an additional empty argument.
    let unparsed = String::deserialize(deserializer)?;
    let unparsed_split = iter::once("").chain(unparsed.split_whitespace());

    InvokeCommand::try_parse_from(unparsed_split).map_err(|err| {
      // Format the error message and remove the "error: " prefix.
      let err_msg = err.apply::<KindFormatter>().to_string();
      serde::de::Error::custom(err_msg.trim_start_matches("error: "))
    })
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, ValueEnum)]
#[clap(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TitleBarVisibility {
  Shown,
  Hidden,
}

#[derive(Args, Clone, Debug, PartialEq, Serialize)]
#[group(required = true, multiple = true)]
pub struct InvokeAdjustBordersCommand {
  #[clap(long, allow_hyphen_values = true)]
  pub top: Option<LengthValue>,

  #[clap(long, allow_hyphen_values = true)]
  pub right: Option<LengthValue>,

  #[clap(long, allow_hyphen_values = true)]
  pub bottom: Option<LengthValue>,

  #[clap(long, allow_hyphen_values = true)]
  pub left: Option<LengthValue>,
}

#[derive(Args, Clone, Debug, PartialEq, Serialize)]
#[group(required = true, multiple = false)]
#[allow(clippy::struct_excessive_bools)]
pub struct InvokeFocusCommand {
  #[clap(long)]
  pub direction: Option<Direction>,

  #[clap(long)]
  pub container_id: Option<Uuid>,

  #[clap(long)]
  pub workspace_in_direction: Option<Direction>,

  #[clap(long)]
  pub workspace: Option<String>,

  #[clap(long)]
  pub monitor: Option<usize>,

  #[clap(long)]
  pub next_active_workspace: bool,

  #[clap(long)]
  pub prev_active_workspace: bool,

  #[clap(long)]
  pub next_workspace: bool,

  #[clap(long)]
  pub prev_workspace: bool,

  #[clap(long)]
  pub next_active_workspace_on_monitor: bool,

  #[clap(long)]
  pub prev_active_workspace_on_monitor: bool,

  #[clap(long)]
  pub recent_workspace: bool,
}

#[derive(Args, Clone, Debug, PartialEq, Serialize)]
#[group(required = true, multiple = false)]
#[allow(clippy::struct_excessive_bools)]
pub struct InvokeMoveCommand {
  /// 窗口移动方向。
  #[clap(long)]
  pub direction: Option<Direction>,

  /// 将窗口移至指定方向的工作区。
  #[clap(long)]
  pub workspace_in_direction: Option<Direction>,

  /// 目标工作区的名称。
  #[clap(long)]
  pub workspace: Option<String>,

  #[clap(long)]
  pub next_active_workspace: bool,

  #[clap(long)]
  pub prev_active_workspace: bool,

  #[clap(long)]
  pub next_workspace: bool,

  #[clap(long)]
  pub prev_workspace: bool,

  #[clap(long)]
  pub next_active_workspace_on_monitor: bool,

  #[clap(long)]
  pub prev_active_workspace_on_monitor: bool,

  #[clap(long)]
  pub recent_workspace: bool,
}

#[derive(Args, Clone, Debug, PartialEq, Serialize)]
#[group(required = true, multiple = true)]
pub struct InvokeResizeCommand {
  #[clap(long, allow_hyphen_values = true)]
  pub width: Option<LengthValue>,

  #[clap(long, allow_hyphen_values = true)]
  pub height: Option<LengthValue>,
}

#[derive(Args, Clone, Debug, PartialEq, Serialize)]
#[group(required = true, multiple = true)]
pub struct SetTransparencyCommand {
  #[clap(long)]
  pub opacity: Option<OpacityValue>,

  #[clap(long, allow_hyphen_values = true)]
  pub opacity_delta: Option<Delta<OpacityValue>>,
}

#[derive(Args, Clone, Debug, PartialEq, Serialize)]
#[group(required = true, multiple = true)]
pub struct InvokePositionCommand {
  #[clap(long, action)]
  pub centered: bool,

  #[clap(long, allow_hyphen_values = true)]
  pub x_pos: Option<i32>,

  #[clap(long, allow_hyphen_values = true)]
  pub y_pos: Option<i32>,
}

#[derive(Args, Clone, Debug, PartialEq, Serialize)]
#[group(required = true, multiple = true)]
pub struct InvokeUpdateWorkspaceConfig {
  #[clap(long, allow_hyphen_values = true)]
  pub name: Option<String>,

  #[clap(long, allow_hyphen_values = true)]
  pub display_name: Option<String>,

  #[clap(long)]
  pub bind_to_monitor: Option<u32>,

  #[clap(long)]
  pub keep_alive: Option<bool>,
}
