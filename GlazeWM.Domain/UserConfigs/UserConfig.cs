using System.Collections.Generic;

namespace GlazeWM.Domain.UserConfigs
{
  public class UserConfig
  {
    public GapsConfig Gaps { get; set; } = new GapsConfig();

    public BarConfig Bar { get; set; } = new BarConfig();

    public List<WorkspaceConfig> Workspaces = new List<WorkspaceConfig>();

    public List<WindowRuleConfig> WindowRules = new List<WindowRuleConfig>();

    public List<KeybindingConfig> Keybindings = new List<KeybindingConfig>();
  }
}
