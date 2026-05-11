const ROOT_CARGO: &str = include_str!("../../../Cargo.toml");
const WM_CARGO: &str = include_str!("../Cargo.toml");

#[test]
fn shell_words_is_registered_as_a_workspace_dependency() {
  assert!(ROOT_CARGO.contains("shell-words = \"1\""));
}

#[test]
fn wm_uses_the_workspace_shell_words_dependency() {
  assert!(WM_CARGO.contains("shell-words = { workspace = true }"));
  assert!(!WM_CARGO.contains("shell-words = \"1\""));
}
