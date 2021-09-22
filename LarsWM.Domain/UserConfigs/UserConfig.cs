using System;
using System.Collections.Generic;

namespace LarsWM.Domain.UserConfigs
{
  public class UserConfig
  {
    public Guid Id = Guid.NewGuid();

    public string ModKey { get; set; } = "Alt";

    public double ResizePercentage { get; set; } = 5;

    /// <summary>
    /// Resize percentage in decimal form.
    /// </summary>
    public double ResizeProportion => ResizePercentage / 100;

    public GapsConfig Gaps { get; set; }

    public BarConfig Bar { get; set; }

    public List<WorkspaceConfig> Workspaces = new List<WorkspaceConfig>();

    public List<WindowRuleConfig> WindowRules = new List<WindowRuleConfig>();

    public List<KeybindingConfig> Keybindings = new List<KeybindingConfig>();
  }
}
