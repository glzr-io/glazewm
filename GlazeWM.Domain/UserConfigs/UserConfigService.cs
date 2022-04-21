using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using GlazeWM.Domain.Windows;

namespace GlazeWM.Domain.UserConfigs
{
  public class UserConfigService
  {
    public UserConfig UserConfig { get; set; } = null;

    /// <summary>
    /// Path to the user's config file.
    /// </summary>
    public string UserConfigPath = Path.Combine(
      Environment.GetFolderPath(Environment.SpecialFolder.UserProfile),
      "./.glaze-wm/config.yaml"
    );

    /// <summary>
    /// Path to the sample user config file.
    /// </summary>
    public string SampleUserConfigPath = Path.Combine(
      Directory.GetCurrentDirectory(),
      "./sample-config.yaml"
    );

    public readonly List<WindowRuleConfig> DefaultWindowRules = GetDefaultWindowRules();

    private static List<WindowRuleConfig> GetDefaultWindowRules()
    {
      var windowRules = new List<WindowRuleConfig>();

      var classNamesToIgnore = new List<string> {
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
      };

      foreach (var className in classNamesToIgnore)
      {
        var windowRule = new WindowRuleConfig()
        {
          MatchClassName = className,
          Command = "ignore",
        };

        windowRules.Add(windowRule);
      }

      var processNamesToIgnore = new List<string> {
        "SearchUI",
        "ShellExperienceHost",
        "LockApp",
        "PeopleExperienceHost",
        "StartMenuExperienceHost",
      };

      foreach (var processName in processNamesToIgnore)
      {
        var windowRule = new WindowRuleConfig()
        {
          MatchProcessName = processName,
          Command = "ignore",
        };

        windowRules.Add(windowRule);
      }

      // Attempt to match Electron apps by getting windows with class name 'Chrome_WidgetWin_1' and
      // the process is not a Chromium-based browser.
      var fixElectronBorderWindowRule = new WindowRuleConfig()
      {
        MatchProcessName = "/^((?!chrome).)*$/",
        MatchClassName = "Chrome_WidgetWin_1",
        Command = "resize borders 0px -7px -7px -7px",
      };

      windowRules.Add(fixElectronBorderWindowRule);

      return windowRules;
    }

    public WorkspaceConfig GetWorkspaceConfigByName(string workspaceName)
    {
      return UserConfig.Workspaces.FirstOrDefault(
        (workspaceConfig) => workspaceConfig.Name == workspaceName
      );
    }

    public IEnumerable<WindowRuleConfig> GetMatchingWindowRules(Window window)
    {
      return UserConfig.WindowRules.Where(rule =>
      {
        if (rule.ProcessNameRegex?.IsMatch(window.ProcessName) == false)
          return false;

        if (rule.ClassNameRegex?.IsMatch(window.ClassName) == false)
          return false;

        if (rule.TitleRegex?.IsMatch(window.Title) == false)
          return false;

        return true;
      });
    }
  }
}
