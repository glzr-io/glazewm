using System.Collections.Generic;
using System.Linq;

namespace GlazeWM.Domain.UserConfigs
{
  public class UserConfig
  {
    public GapsConfig Gaps { get; set; } = new GapsConfig();

    public GeneralConfig General { get; set; } = new GeneralConfig();

    public BarConfig Bar { get; set; } = new();

    public List<MultiBarConfig> Bars { get; set; } = new();

    public List<BarConfig> BarConfigs => Bars.Count == 0
      ? new List<BarConfig> { Bar }
      : Bars.Cast<BarConfig>().ToList();

    public List<WorkspaceConfig> Workspaces { get; set; } = new();

    public List<WindowRuleConfig> WindowRules { get; set; } = new();

    public List<KeybindingConfig> Keybindings { get; set; } = new();

    public List<BindingMode> BindingModes { get; set; } = new();
  }
}
