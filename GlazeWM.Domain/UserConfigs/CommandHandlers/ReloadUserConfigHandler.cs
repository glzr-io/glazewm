using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.UserConfigs.Commands;
using GlazeWM.Domain.UserConfigs.Events;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.UserConfigs.CommandHandlers
{
  internal sealed class ReloadUserConfigHandler : ICommandHandler<ReloadUserConfigCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;
    private readonly UserConfigService _userConfigService;
    private readonly WindowService _windowService;

    public ReloadUserConfigHandler(
      Bus bus,
      ContainerService containerService,
      UserConfigService userConfigService,
      WindowService windowService)
    {
      _bus = bus;
      _containerService = containerService;
      _userConfigService = userConfigService;
      _windowService = windowService;
    }

    public CommandResponse Handle(ReloadUserConfigCommand command)
    {
      // Re-evaluate user config file and set its values in state.
      _bus.Invoke(new EvaluateUserConfigCommand());

      _bus.Invoke(new UpdateWorkspacesFromConfigCommand(_userConfigService.WorkspaceConfigs));

      foreach (var window in _windowService.GetWindows())
      {
        var windowRules = _userConfigService.GetMatchingWindowRules(window);
        var windowRuleCommands = windowRules
          .SelectMany(rule => rule.CommandList)
          .Select(CommandParsingService.FormatCommand);

        // Run matching window rules.
        _bus.Invoke(new RunWithSubjectContainerCommand(windowRuleCommands, window));
      }

      // Redraw full container tree.
      _containerService.ContainersToRedraw.Add(_containerService.ContainerTree);
      _bus.Invoke(new RedrawContainersCommand());

      _bus.Emit(new UserConfigReloadedEvent());

      return CommandResponse.Ok;
    }
  }
}
