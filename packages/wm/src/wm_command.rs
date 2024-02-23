pub enum WmCommand {
  CloseWindow(dyn Container),
  ExitWm,
  Focus(dyn Container, Direction),
  FocusWorkspace(dyn Container, String),
  IgnoreWindow(dyn Container),
  MoveWindow(dyn Container, Direction),
  MoveWorkspace(dyn Container, Direction),
  Noop,
  ReloadConfig,
  SetTilingDirection(dyn Container, TilingDirection),
  ToggleTilingDirection(dyn Container),
}

impl WmCommand {
  pub fn from_str(
    unparsed: &str,
    subject_container: dyn Container,
  ) -> Result<Command> {
    let parts: Vec<&str> = unparsed.split_whitespace().collect();

    let command = match parts.as_slice() {
      ["close_window"] => WmCommand::CloseWindow(subject_container),
      ["exit_wm"] => WmCommand::ExitWm,
      ["focus", "left"] => {
        WmCommand::Focus(subject_container, Direction::Left)
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
        WmCommand::FocusWorkspace(subject_container, Direction::Down)
      }
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
      ["reload_config"] => WmCommand::ReloadConfig,
      _ => Err("Not a known command."),
    };

    Ok(command)
  }
}
