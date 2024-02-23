pub enum IpcMessage {
  QueryMonitors,
  QueryWorkspaces,
  QueryWindows,
  QueryFocusedContainer,
  QueryBindingMode,
  InvokeCommand,
  SubscribeEvent,
}
