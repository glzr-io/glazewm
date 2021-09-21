using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.UserConfigs;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;
using System.Collections.Generic;
using System.Linq;

namespace LarsWM.Domain.Windows.CommandHandlers
{
  class AddInitialWindowsHandler : ICommandHandler<AddInitialWindowsCommand>
  {
    private Bus _bus;
    private UserConfigService _userConfigService;
    private MonitorService _monitorService;
    private WindowService _windowService;
    private CommandParsingService _commandParsingService;

    public AddInitialWindowsHandler(
        Bus bus,
        UserConfigService userConfigService,
        MonitorService monitorService,
        WindowService windowService, CommandParsingService commandParsingService)
    {
      _bus = bus;
      _userConfigService = userConfigService;
      _monitorService = monitorService;
      _windowService = windowService;
      _commandParsingService = commandParsingService;
    }

    public CommandResponse Handle(AddInitialWindowsCommand command)
    {
      var manageableWindows = _windowService.GetAllWindowHandles()
        .Select(handle => new Window(handle))
        .Where(window => window.IsManageable);

      foreach (var window in manageableWindows)
      {
        var matchingWindowRules = GetMatchingWindowRules(window);

        var commandStrings = matchingWindowRules
          .SelectMany(rule => rule.CommandStrings)
          .Select(commandString => _commandParsingService.FormatCommand(commandString));

        if (commandStrings.Contains("ignore"))
          continue;

        // Get workspace that encompasses most of the window.
        var targetMonitor = _monitorService.GetMonitorFromUnmanagedHandle(window.Hwnd);
        var targetWorkspace = targetMonitor.DisplayedWorkspace;

        _bus.Invoke(new AttachContainerCommand(targetMonitor.DisplayedWorkspace, window));
      }

      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }

    private List<WindowRuleConfig> GetMatchingWindowRules(Window window)
    {
      return _userConfigService.UserConfig.WindowRules
        .Where(rule =>
        {
          if (rule.ProcessNameRegex != null && !rule.ProcessNameRegex.IsMatch(window.Process.ProcessName))
            return false;

          if (rule.ClassNameRegex != null && !rule.ClassNameRegex.IsMatch(window.ClassName))
            return false;

          if (rule.TitleRegex != null && !rule.TitleRegex.IsMatch(window.Title))
            return false;

          return true;
        })
        .ToList();
    }
  }
}
