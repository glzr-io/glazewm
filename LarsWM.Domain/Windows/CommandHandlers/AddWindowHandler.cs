using System.Collections.Generic;
using System.Linq;
using System.Text.RegularExpressions;
using LarsWM.Domain.Containers;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.UserConfigs;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Domain.Workspaces;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Windows.CommandHandlers
{
  class AddWindowHandler : ICommandHandler<AddWindowCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;
    private WindowService _windowService;
    private MonitorService _monitorService;
    private UserConfigService _userConfigService;
    private CommandParsingService _commandParsingService;

    public AddWindowHandler(Bus bus, WindowService windowService, MonitorService monitorService, ContainerService containerService, UserConfigService userConfigService, CommandParsingService commandParsingService)
    {
      _bus = bus;
      _windowService = windowService;
      _monitorService = monitorService;
      _containerService = containerService;
      _userConfigService = userConfigService;
      _commandParsingService = commandParsingService;
    }

    public CommandResponse Handle(AddWindowCommand command)
    {
      var window = new Window(command.WindowHandle);

      if (!window.IsManageable)
        return CommandResponse.Ok;

      var matchingWindowRules = GetMatchingWindowRules(window);

      var commandStrings = matchingWindowRules
        .SelectMany(rule => rule.CommandStrings)
        .Select(commandString => _commandParsingService.FormatCommand(commandString));

      // Avoid managing a window if a window rule uses 'ignore' command.
      if (commandStrings.Contains("ignore"))
        return CommandResponse.Ok;

      var focusedContainer = _containerService.FocusedContainer;

      // If the focused container is a workspace, attach the window as a child of the workspace.
      if (focusedContainer is Workspace)
        _bus.Invoke(new AttachContainerCommand(focusedContainer as Workspace, window));

      // Attach the window as a sibling next to the focused window.
      else
        _bus.Invoke(new AttachContainerCommand(
          focusedContainer.Parent as SplitContainer, window, focusedContainer.Index + 1
        ));

      // Set focus to newly added window in case it has not been focused automatically.
      _bus.Invoke(new FocusWindowCommand(window));

      var parsedCommands = commandStrings
        .Select(commandString => _commandParsingService.ParseCommand(commandString));

      foreach (var parsedCommand in parsedCommands)
        _bus.Invoke((dynamic)parsedCommand);

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
