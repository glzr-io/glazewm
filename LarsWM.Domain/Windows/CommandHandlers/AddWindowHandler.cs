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

    public AddWindowHandler(Bus bus, WindowService windowService, MonitorService monitorService, ContainerService containerService, UserConfigService userConfigService)
    {
      _bus = bus;
      _windowService = windowService;
      _monitorService = monitorService;
      _containerService = containerService;
      _userConfigService = userConfigService;
    }

    public CommandResponse Handle(AddWindowCommand command)
    {
      var window = new Window(command.WindowHandle);

      if (!window.IsManageable)
        return CommandResponse.Ok;

      var matchedWindowRules = _userConfigService.UserConfig.WindowRules
        .Where(rule => rule.ClassNameRegex.IsMatch(window.ClassName))
        .Where(rule => rule.ProcessNameRegex.IsMatch(window.Process.ProcessName))
        .Where(rule => rule.TitleRegex.IsMatch(window.Title));

      var focusedContainer = _containerService.FocusedContainer;

      // If the focused container is a workspace, attach the window as a child of the
      // workspace. Otherwise, attach the window as a sibling to the focused window.
      if (focusedContainer is Workspace)
        _bus.Invoke(new AttachContainerCommand(focusedContainer as Workspace, window));
      else
        _bus.Invoke(new AttachContainerCommand(focusedContainer.Parent as SplitContainer, window));

      _bus.Invoke(new RedrawContainersCommand());

      // Set focus to newly added window in case it has not been focused automatically.
      _bus.Invoke(new FocusWindowCommand(window));

      return CommandResponse.Ok;
    }
  }
}
