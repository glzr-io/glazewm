use anyhow::{bail, Context};
use tokio::sync::mpsc::{self};
use tracing::warn;
use uuid::Uuid;
use wm_common::{
  FloatingStateConfig, FullscreenStateConfig, InvokeCommand, LengthValue,
  RectDelta, TitleBarVisibility, WindowState, WmEvent,
};
use wm_platform::PlatformEvent;

use crate::{
  commands::{
    container::{
      focus_in_direction, set_tiling_direction, toggle_tiling_direction,
    },
    general::{
      cycle_focus, disable_binding_mode, enable_binding_mode,
      platform_sync, reload_config, shell_exec, toggle_pause,
    },
    monitor::focus_monitor,
    window::{
      ignore_window, move_window_in_direction, move_window_to_workspace,
      resize_window, set_window_position, set_window_size,
      update_window_state, WindowPositionTarget,
    },
    workspace::{focus_workspace, move_workspace_in_direction},
  },
  events::{
    handle_display_settings_changed, handle_mouse_move,
    handle_window_destroyed, handle_window_focused, handle_window_hidden,
    handle_window_location_changed, handle_window_minimize_ended,
    handle_window_minimized, handle_window_moved_or_resized_end,
    handle_window_moved_or_resized_start, handle_window_shown,
    handle_window_title_changed,
  },
  models::{Container, WorkspaceTarget},
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub struct WindowManager {
  pub event_rx: mpsc::UnboundedReceiver<WmEvent>,
  pub exit_rx: mpsc::UnboundedReceiver<()>,
  pub state: WmState,
}

impl WindowManager {
  pub fn new(config: &mut UserConfig) -> anyhow::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let (exit_tx, exit_rx) = mpsc::unbounded_channel();

    let mut state = WmState::new(event_tx, exit_tx);
    state.populate(config)?;

    Ok(Self {
      event_rx,
      exit_rx,
      state,
    })
  }

  pub fn process_event(
    &mut self,
    event: PlatformEvent,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    let state = &mut self.state;

    match event {
      PlatformEvent::DisplaySettingsChanged => {
        handle_display_settings_changed(state, config)
      }
      PlatformEvent::KeybindingTriggered(kb_config) => {
        self.process_commands(&kb_config.commands, None, config)?;

        // Return early since we don't want to redraw twice.
        return Ok(());
      }
      PlatformEvent::MouseMove(event) => {
        handle_mouse_move(&event, state, config)
      }
      PlatformEvent::WindowDestroyed(window) => {
        handle_window_destroyed(&window, state)
      }
      PlatformEvent::WindowFocused(window) => {
        handle_window_focused(&window, state, config)
      }
      PlatformEvent::WindowHidden(window) => {
        handle_window_hidden(&window, state)
      }
      PlatformEvent::WindowLocationChanged(window) => {
        handle_window_location_changed(&window, state, config)
      }
      PlatformEvent::WindowMinimized(window) => {
        handle_window_minimized(&window, state, config)
      }
      PlatformEvent::WindowMinimizeEnded(window) => {
        handle_window_minimize_ended(&window, state, config)
      }
      PlatformEvent::WindowMovedOrResizedEnd(window) => {
        handle_window_moved_or_resized_end(&window, state, config)
      }
      PlatformEvent::WindowMovedOrResizedStart(window) => {
        handle_window_moved_or_resized_start(&window, state);
        Ok(())
      }
      PlatformEvent::WindowShown(window) => {
        handle_window_shown(window, state, config)
      }
      PlatformEvent::WindowTitleChanged(window) => {
        handle_window_title_changed(&window, state, config)
      }
    }?;

    if state.pending_sync.has_changes() {
      platform_sync(state, config)?;
    }

    Ok(())
  }

  pub fn process_commands(
    &mut self,
    commands: &Vec<InvokeCommand>,
    subject_container_id: Option<Uuid>,
    config: &mut UserConfig,
  ) -> anyhow::Result<Uuid> {
    let state = &mut self.state;

    // Get the container to run WM commands with.
    let subject_container = match subject_container_id {
      Some(id) => state.container_by_id(id).with_context(|| {
        format!("No container found with the given ID '{id}'.")
      })?,
      None => state
        .focused_container()
        .context("No subject container for command.")?,
    };

    let new_subject_container_id = WindowManager::run_commands(
      commands,
      subject_container,
      state,
      config,
    )?;

    if state.pending_sync.has_changes() {
      platform_sync(state, config)?;
    }

    Ok(new_subject_container_id)
  }

  pub fn run_commands(
    commands: &Vec<InvokeCommand>,
    subject_container: Container,
    state: &mut WmState,
    config: &mut UserConfig,
  ) -> anyhow::Result<Uuid> {
    let mut current_subject_container = subject_container;

    for command in commands {
      WindowManager::run_command(
        command,
        current_subject_container.clone(),
        state,
        config,
      )?;

      // Update the subject container in case the container type changes.
      // For example, when going from a tiling to a floating window.
      current_subject_container =
        if current_subject_container.is_detached() {
          match state.container_by_id(current_subject_container.id()) {
            Some(container) => container,
            None => break,
          }
        } else {
          current_subject_container
        }
    }

    Ok(current_subject_container.id())
  }

  #[allow(clippy::too_many_lines)]
  pub fn run_command(
    command: &InvokeCommand,
    subject_container: Container,
    state: &mut WmState,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    if subject_container.is_detached() {
      bail!("Cannot run command because subject container is detached.");
    }

    match &command {
      InvokeCommand::AdjustBorders(args) => {
        match subject_container.as_window_container() {
          Ok(window) => {
            let args = args.clone();
            let border_delta = RectDelta::new(
              args.left.unwrap_or(LengthValue::from_px(0)),
              args.top.unwrap_or(LengthValue::from_px(0)),
              args.right.unwrap_or(LengthValue::from_px(0)),
              args.bottom.unwrap_or(LengthValue::from_px(0)),
            );

            window.set_border_delta(border_delta);
            state.pending_sync.containers_to_redraw.push(window.into());

            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::Close => {
        match subject_container.as_window_container() {
          Ok(window) => {
            // Window handle might no longer be valid here.
            if let Err(err) = window.native().close() {
              warn!("Failed to close window: {:?}", err);
            }

            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::Focus(args) => {
        if let Some(direction) = &args.direction {
          focus_in_direction(&subject_container, direction, state)?;
        }

        if let Some(name) = &args.workspace {
          focus_workspace(
            WorkspaceTarget::Name(name.to_string()),
            state,
            config,
          )?;
        }

        if let Some(monitor_index) = &args.monitor {
          focus_monitor(*monitor_index, state, config)?;
        }

        if args.next_active_workspace {
          focus_workspace(WorkspaceTarget::NextActive, state, config)?;
        }

        if args.prev_active_workspace {
          focus_workspace(WorkspaceTarget::PreviousActive, state, config)?;
        }

        if args.next_workspace {
          focus_workspace(WorkspaceTarget::Next, state, config)?;
        }

        if args.prev_workspace {
          focus_workspace(WorkspaceTarget::Previous, state, config)?;
        }

        if args.recent_workspace {
          focus_workspace(WorkspaceTarget::Recent, state, config)?;
        }

        Ok(())
      }
      InvokeCommand::Ignore => {
        match subject_container.as_window_container() {
          Ok(window) => ignore_window(window, state),
          _ => Ok(()),
        }
      }
      InvokeCommand::Move(args) => {
        match subject_container.as_window_container() {
          Ok(window) => {
            if let Some(direction) = &args.direction {
              move_window_in_direction(
                window.clone(),
                direction,
                state,
                config,
              )?;
            };

            if let Some(name) = &args.workspace {
              move_window_to_workspace(
                window.clone(),
                WorkspaceTarget::Name(name.to_string()),
                state,
                config,
              )?;
            }

            if args.next_active_workspace {
              move_window_to_workspace(
                window.clone(),
                WorkspaceTarget::NextActive,
                state,
                config,
              )?;
            }

            if args.prev_active_workspace {
              move_window_to_workspace(
                window.clone(),
                WorkspaceTarget::PreviousActive,
                state,
                config,
              )?;
            }

            if args.next_workspace {
              move_window_to_workspace(
                window.clone(),
                WorkspaceTarget::Next,
                state,
                config,
              )?;
            }

            if args.prev_workspace {
              move_window_to_workspace(
                window.clone(),
                WorkspaceTarget::Previous,
                state,
                config,
              )?;
            }

            if args.recent_workspace {
              move_window_to_workspace(
                window,
                WorkspaceTarget::Recent,
                state,
                config,
              )?;
            }

            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::MoveWorkspace { direction } => {
        let workspace =
          subject_container.workspace().context("No workspace.")?;

        move_workspace_in_direction(&workspace, direction, state, config)
      }
      InvokeCommand::Position(args) => {
        match subject_container.as_window_container() {
          Ok(window) => {
            if args.centered {
              set_window_position(
                window,
                &WindowPositionTarget::Centered,
                state,
              )
            } else {
              set_window_position(
                window,
                &WindowPositionTarget::Coordinates(args.x_pos, args.y_pos),
                state,
              )
            }
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::Resize(args) => {
        match subject_container.as_window_container() {
          Ok(window) => resize_window(
            &window,
            args.width.clone(),
            args.height.clone(),
            state,
          ),
          _ => Ok(()),
        }
      }
      InvokeCommand::SetFloating {
        centered,
        shown_on_top,
        x_pos,
        y_pos,
        width,
        height,
      } => match subject_container.as_window_container() {
        Ok(window) => {
          let floating_defaults =
            &config.value.window_behavior.state_defaults.floating;
          let centered = centered.unwrap_or(floating_defaults.centered);

          let window = update_window_state(
            window.clone(),
            WindowState::Floating(FloatingStateConfig {
              centered,
              shown_on_top: shown_on_top
                .unwrap_or(floating_defaults.shown_on_top),
            }),
            state,
            config,
          )?;

          // Allow size and position to be set if window has not previously
          // been manually placed.
          if !window.has_custom_floating_placement() {
            if width.is_some() || height.is_some() {
              set_window_size(
                window.clone(),
                width.clone(),
                height.clone(),
                state,
              )?;
            }

            if centered {
              set_window_position(
                window,
                &WindowPositionTarget::Centered,
                state,
              )?;
            } else if x_pos.is_some() || y_pos.is_some() {
              set_window_position(
                window,
                &WindowPositionTarget::Coordinates(*x_pos, *y_pos),
                state,
              )?;
            }
          }

          Ok(())
        }
        _ => Ok(()),
      },
      InvokeCommand::SetFullscreen {
        maximized,
        shown_on_top,
      } => match subject_container.as_window_container() {
        Ok(window) => {
          let fullscreen_defaults =
            &config.value.window_behavior.state_defaults.fullscreen;

          update_window_state(
            window.clone(),
            WindowState::Fullscreen(FullscreenStateConfig {
              maximized: maximized
                .unwrap_or(fullscreen_defaults.maximized),
              shown_on_top: shown_on_top
                .unwrap_or(fullscreen_defaults.shown_on_top),
            }),
            state,
            config,
          )?;

          Ok(())
        }
        _ => Ok(()),
      },
      InvokeCommand::SetMinimized => {
        match subject_container.as_window_container() {
          Ok(window) => {
            update_window_state(
              window.clone(),
              WindowState::Minimized,
              state,
              config,
            )?;

            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::SetTiling => {
        match subject_container.as_window_container() {
          Ok(window) => {
            update_window_state(
              window,
              WindowState::Tiling,
              state,
              config,
            )?;

            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::SetTitleBarVisibility { visibility } => {
        match subject_container.as_window_container() {
          Ok(window) => {
            _ = window.native().set_title_bar_visibility(
              *visibility == TitleBarVisibility::Shown,
            );
            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::SetOpacity { opacity } => {
        match subject_container.as_window_container() {
          Ok(window) => {
            _ = window.native().set_opacity(opacity);
            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::ShellExec {
        hide_window,
        command,
      } => shell_exec(&command.join(" "), *hide_window),
      InvokeCommand::Size(args) => {
        match subject_container.as_window_container() {
          Ok(window) => set_window_size(
            window,
            args.width.clone(),
            args.height.clone(),
            state,
          ),
          _ => Ok(()),
        }
      }
      InvokeCommand::ToggleFloating {
        centered,
        shown_on_top,
      } => match subject_container.as_window_container() {
        Ok(window) => {
          let floating_defaults =
            &config.value.window_behavior.state_defaults.floating;

          let centered = centered.unwrap_or(floating_defaults.centered);
          let target_state = WindowState::Floating(FloatingStateConfig {
            centered,
            shown_on_top: shown_on_top
              .unwrap_or(floating_defaults.shown_on_top),
          });

          let window = update_window_state(
            window.clone(),
            window.toggled_state(target_state, config),
            state,
            config,
          )?;

          if !window.has_custom_floating_placement() && centered {
            set_window_position(
              window,
              &WindowPositionTarget::Centered,
              state,
            )?;
          }

          Ok(())
        }
        _ => Ok(()),
      },
      InvokeCommand::ToggleFullscreen {
        maximized,
        shown_on_top,
      } => match subject_container.as_window_container() {
        Ok(window) => {
          let fullscreen_defaults =
            &config.value.window_behavior.state_defaults.fullscreen;

          let target_state =
            WindowState::Fullscreen(FullscreenStateConfig {
              maximized: maximized
                .unwrap_or(fullscreen_defaults.maximized),
              shown_on_top: shown_on_top
                .unwrap_or(fullscreen_defaults.shown_on_top),
            });

          update_window_state(
            window.clone(),
            window.toggled_state(target_state, config),
            state,
            config,
          )?;

          Ok(())
        }
        _ => Ok(()),
      },
      InvokeCommand::ToggleMinimized => {
        match subject_container.as_window_container() {
          Ok(window) => {
            update_window_state(
              window.clone(),
              window.toggled_state(WindowState::Minimized, config),
              state,
              config,
            )?;

            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::ToggleTiling => {
        match subject_container.as_window_container() {
          Ok(window) => {
            update_window_state(
              window.clone(),
              window.toggled_state(WindowState::Tiling, config),
              state,
              config,
            )?;

            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::ToggleTilingDirection => {
        toggle_tiling_direction(subject_container, state, config)
      }
      InvokeCommand::SetTilingDirection { tiling_direction } => {
        set_tiling_direction(
          subject_container,
          state,
          config,
          tiling_direction,
        )
      }
      InvokeCommand::WmCycleFocus {
        omit_fullscreen,
        omit_minimized,
      } => cycle_focus(*omit_fullscreen, *omit_minimized, state, config),
      InvokeCommand::WmDisableBindingMode { name } => {
        disable_binding_mode(name, state);
        Ok(())
      }
      InvokeCommand::WmEnableBindingMode { name } => {
        enable_binding_mode(name, state, config)
      }
      InvokeCommand::WmExit => state.emit_exit(),
      InvokeCommand::WmRedraw => {
        let root_container = state.root_container.clone();
        state
          .pending_sync
          .containers_to_redraw
          .push(root_container.into());

        Ok(())
      }
      InvokeCommand::WmReloadConfig => reload_config(state, config),
      InvokeCommand::WmTogglePause => {
        toggle_pause(state);
        Ok(())
      }
    }
  }
}
