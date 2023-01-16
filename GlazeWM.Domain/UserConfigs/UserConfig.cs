using System.Collections.Generic;

namespace GlazeWM.Domain.UserConfigs
{
  public class UserConfig
  {
    public GapsConfig Gaps { get; set; } = new GapsConfig();

    public GeneralConfig General { get; set; } = new GeneralConfig();

    public BarConfig Bar { get; set; } = new BarConfig();

    public List<WorkspaceConfig> Workspaces { get; set; } = new();

    public List<WindowRuleConfig> WindowRules { get; set; } = new();

    public List<KeybindingConfig> Keybindings { get; set; } = new();

    public List<BindingMode> BindingModes { get; set; } = new();
  }
}
