using System.Collections.Generic;

namespace LarsWM.Domain.UserConfigs
{
  // TODO: Merge with `UserConfig` class.
  class UserConfigFileDto
  {
    public double ResizePercentage { get; set; } = 5;
    public GapsConfig Gaps { get; set; }
    public BarConfig Bar { get; set; }
    public List<WorkspaceConfig> Workspaces { get; set; }
    public List<KeybindingConfig> Keybindings { get; set; }
  }

  // TODO: Move within `UserConfig`.
  public class GapsConfig
  {
    public int InnerGap { get; set; } = 20;
    public int OuterGap { get; set; } = 20;
  }

  // TODO: Move within `UserConfig`.
  public class BarConfig
  {
    public int Height { get; set; } = 50;
  }

  // TODO: Move within `UserConfig`.
  public class WorkspaceConfig
  {
    public string Name { get; set; }
    public string BindToMonitor { get; set; } = null;
    public string CustomDisplayName { get; set; } = null;
    public bool KeepAlive { get; set; } = false;
  }

  // TODO: Move within `UserConfig`.
  public class KeybindingConfig
  {
    public string Command { get; set; }
    public List<string> Bindings { get; set; }
  }
}
