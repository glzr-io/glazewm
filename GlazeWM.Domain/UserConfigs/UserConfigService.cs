using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using GlazeWM.Domain.Windows;

namespace GlazeWM.Domain.UserConfigs
{
  public class UserConfigService
  {
    public UserConfig UserConfig { get; set; }

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

      var classNamesToFloat = new List<string> {
        // Dialog shown when moving and deleting files.
        "OperationStatusWindow",
      };

      foreach (var className in classNamesToFloat)
      {
        var windowRule = new WindowRuleConfig()
        {
          MatchClassName = className,
          Command = "set floating",
        };

        windowRules.Add(windowRule);
      }

      var chromiumBrowserProcessNames = new List<string> {
        "chrome",
        "msedge",
        "opera",
        "vivaldi",
        "brave",
      };

      // Electron apps do not have invisible borders and are thus over-corrected by the default
      // border fix. To match these apps, get windows with the class name 'Chrome_WidgetWin_1' that
      // are not Chromium-based browsers (since the browser windows do have invisble borders).
      var resizeElectronBorderWindowRule = new WindowRuleConfig()
      {
        MatchProcessName = $"/^(?!({string.Join("|", chromiumBrowserProcessNames)})$)/",
        MatchClassName = "Chrome_WidgetWin_1",
        Command = "resize borders 0px -7px -7px -7px",
      };

      windowRules.Add(resizeElectronBorderWindowRule);

      return windowRules;
    }

    public WorkspaceConfig GetWorkspaceConfigByName(string workspaceName)
    {
      return UserConfig.Workspaces.Find(
        (workspaceConfig) => workspaceConfig.Name == workspaceName
      );
    }

    public IEnumerable<WindowRuleConfig> GetMatchingWindowRules(Window window)
    {
      return UserConfig.WindowRules.Where(rule =>
      {
        return rule.ProcessNameRegex?.IsMatch(window.ProcessName) != false
          && rule.ClassNameRegex?.IsMatch(window.ClassName) != false
          && (rule.TitleRegex?.IsMatch(window.Title)) != false;
      });
    }
  }
}
