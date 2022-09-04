using System.Linq;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.UserConfigs.Commands;
using GlazeWM.Domain.UserConfigs.Events;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.UserConfigs.CommandHandlers
{
  internal class ReloadUserConfigHandler : ICommandHandler<ReloadUserConfigCommand>
  {
    private readonly Bus _bus;
    private readonly CommandParsingService _commandParsingService;
    private readonly ContainerService _containerService;
    private readonly UserConfigService _userConfigService;
    private readonly WindowService _windowService;

    public ReloadUserConfigHandler(
      Bus bus,
      CommandParsingService commandParsingService,
      ContainerService containerService,
      UserConfigService userConfigService,
      WindowService windowService)
    {
      _bus = bus;
      _commandParsingService = commandParsingService;
      _containerService = containerService;
      _userConfigService = userConfigService;
      _windowService = windowService;
    }

    public CommandResponse Handle(ReloadUserConfigCommand command)
    {
      // Re-evaluate user config file and set its values in state.
      _bus.Invoke(new EvaluateUserConfigCommand());

      _bus.Invoke(new UpdateWorkspacesFromConfigCommand(_userConfigService.UserConfig.Workspaces));

      foreach (var window in _windowService.GetWindows())
      {
        // TODO: Create `RunWindowRulesCommand(Window window, List<WindowRule> windowRules).
        var matchingWindowRules = _userConfigService.GetMatchingWindowRules(window);

        var commandStrings = matchingWindowRules
          .SelectMany(rule => rule.CommandList)
          .Select(commandString => CommandParsingService.FormatCommand(commandString));

        var parsedCommands = commandStrings
          .Select(commandString => _commandParsingService.ParseCommand(commandString, window))
          .ToList();

        // Invoke commands in the matching window rules. Use `dynamic` to resolve the command type
        // at runtime and allow multiple dispatch.
        foreach (var parsedCommand in parsedCommands)
          _bus.Invoke((dynamic)parsedCommand);
      }

      // Redraw full container tree.
      _containerService.ContainersToRedraw.Add(_containerService.ContainerTree);
      _bus.Invoke(new RedrawContainersCommand());

      _bus.RaiseEvent(new UserConfigReloadedEvent());

      return CommandResponse.Ok;
    }
  }
}
