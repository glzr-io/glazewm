using System.Collections.Generic;

namespace GlazeWM.Domain.UserConfigs
{
  public class UserConfig
  {
    public GapsConfig Gaps { get; set; } = new GapsConfig();

    public BarConfig Bar { get; set; } = new BarConfig();

    public List<WorkspaceConfig> Workspaces = new();

    public List<WindowRuleConfig> WindowRules = new();

    public List<KeybindingConfig> Keybindings = new();
  }
}
