using System;
using System.Collections.Generic;

namespace LarsWM.Domain.UserConfigs
{
  public class UserConfig
  {
    public Guid Id = Guid.NewGuid();

    // TODO: Allow regular expressions.
    // eg. for WMP's "now playing" toolbar: StartsWith("WMP9MediaBarFlyout"))
    public List<string> WindowClassesToIgnore = new List<string> {
      // Tray on primary screen.
      "Shell_TrayWnd",
      // Trays on secondary screens.
      "Shell_SecondaryTrayWnd",
      // Task manager.
      "TaskManagerWindow",
      // Microsoft Text Framework service IME.
      "MSCTFIME UI",
      // Desktop window (holds wallpaper & desktop icons).
      "SHELLDLL_DefView",
      // Background for lock screen.
      "LockScreenBackstopFrame",
      // Windows 10 shell.
      "Progman",
      // Windows 7 open Start Menu.
      "DV2ControlHost",
      // Windows 8 charm bar.
      "Shell_CharmWindow",

      /*
       * Consider adding:
       * "MsgrIMEWindowClass", // Window live messenger notification.
       * "SysShadow", // Windows live messenger shadow-hack.
       * "Button", // UI component, e.g. Start Menu button.
       * "Frame Alternate Owner", // Edge.
       * "MultitaskingViewFrame", // The original Win + Tab view.
       */
    };

    public List<string> ProcessNamesToIgnore = new List<string> {
      "SearchUI",
      "ShellExperienceHost",
      "LockApp",
      "PeopleExperienceHost",
      "StartMenuExperienceHost",
    };

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
