pub enum WmCommand {
  CloseWindow(Container),
  ExitWm,
  FocusInDirection(Container, Direction),
  FocusRecentWorkspace,
  FocusWorkspaceInSequence,
  FocusWorkspace(Container, String),
  IgnoreWindow(Container),
  MoveWindow(Container, Direction),
  MoveWorkspace(Container, Direction),
  Noop,
  Redraw,
  ReloadConfig,
  SetTilingDirection(Container, TilingDirection),
  ToggleTilingDirection(Container),
}

impl WmCommand {
  pub fn from_str(
    unparsed: &str,
    subject_container: Container,
  ) -> Result<Self> {
    let parts: Vec<&str> = unparsed.split_whitespace().collect();

    let command = match parts.as_slice() {
      ["close_window"] => WmCommand::CloseWindow(subject_container),
      ["exit_wm"] => WmCommand::ExitWm,
      ["focus", "left"] => {
        WmCommand::FocusInDirection(subject_container, Direction::Left)
      }
      ["focus", "right"] => {
        WmCommand::Focus(subject_container, Direction::Right)
      }
      ["focus", "up"] => {
        WmCommand::Focus(subject_container, Direction::Up)
      }
      ["focus", "down"] => {
        WmCommand::Focus(subject_container, Direction::Down)
      }
      ["focus_workspace", name] => {
        WmCommand::FocusWorkspace(subject_container, name)
      }
      ["ignore_window"] => WmCommand::IgnoreWindow(subject_container),
      ["move_window", "left"] => {
        WmCommand::MoveWindow(subject_container, Direction::Left)
      }
      ["move_window", "right"] => {
        WmCommand::MoveWindow(subject_container, Direction::Right)
      }
      ["move_window", "up"] => {
        WmCommand::MoveWindow(subject_container, Direction::Up)
      }
      ["move_window", "down"] => {
        WmCommand::MoveWindow(subject_container, Direction::Down)
      }
      ["move_workspace", "left"] => {
        WmCommand::MoveWorkspace(subject_container, Direction::Left)
      }
      ["move_workspace", "right"] => {
        WmCommand::MoveWorkspace(subject_container, Direction::Right)
      }
      ["move_workspace", "up"] => {
        WmCommand::MoveWorkspace(subject_container, Direction::Up)
      }
      ["move_workspace", "down"] => {
        WmCommand::MoveWorkspace(subject_container, Direction::Down)
      }
      ["noop"] => WmCommand::Noop,
      ["redraw"] => WmCommand::Redraw,
      ["reload_config"] => WmCommand::ReloadConfig,
      ["set_tiling_direction", "horizontal"] => {
        WmCommand::SetTilingDirection(
          subject_container,
          TilingDirection::Horizontal,
        )
      }
      ["set_tiling_direction", "vertical"] => {
        WmCommand::SetTilingDirection(
          subject_container,
          TilingDirection::Vertical,
        )
      }
      ["toggle_tiling_direction", "vertical"] => {
        WmCommand::ToggleTilingDirection(subject_container)
      }
      _ => Err("Not a known command."),
    };

    Ok(command)
  }
}
