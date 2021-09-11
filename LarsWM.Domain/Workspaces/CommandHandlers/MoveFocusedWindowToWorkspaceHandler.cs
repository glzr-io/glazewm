using LarsWM.Infrastructure.Bussing;
using LarsWM.Domain.Workspaces.Commands;
using LarsWM.Domain.Containers;
using LarsWM.Domain.Windows;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Windows.Commands;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Domain.Workspaces.CommandHandlers
{
  class MoveFocusedWindowToWorkspaceHandler : ICommandHandler<MoveFocusedWindowToWorkspaceCommand>
  {
    private Bus _bus;
    private WorkspaceService _workspaceService;
    private MonitorService _monitorService;
    private ContainerService _containerService;

    public MoveFocusedWindowToWorkspaceHandler(Bus bus, WorkspaceService workspaceService, MonitorService monitorService, ContainerService containerService)
    {
      _bus = bus;
      _workspaceService = workspaceService;
      _monitorService = monitorService;
      _containerService = containerService;
    }

    public CommandResponse Handle(MoveFocusedWindowToWorkspaceCommand command)
    {
      var workspaceName = command.WorkspaceName;

      var focusedWindow = _containerService.FocusedContainer as Window;
      var foregroundWindow = GetForegroundWindow();

      // Ignore cases where focused container is not a window or not in foreground.
      if (focusedWindow == null || foregroundWindow != focusedWindow.Hwnd)
        return CommandResponse.Ok;

      _bus.Invoke(new FocusWorkspaceCommand(workspaceName));

      var currentWorkspace = _workspaceService.GetWorkspaceFromChildContainer(focusedWindow);
      var workspaceToFocus = _workspaceService.GetActiveWorkspaceByName(workspaceName);

      var insertionTarget = workspaceToFocus.LastFocusedDescendant;

      if (insertionTarget == null)
        _bus.Invoke(new AttachContainerCommand(workspaceToFocus, focusedWindow));
      else
        _bus.Invoke(new AttachContainerCommand(insertionTarget.Parent as SplitContainer, focusedWindow, insertionTarget.Index));

      if (currentWorkspace.IsDisplayed)
        _containerService.SplitContainersToRedraw.Add(currentWorkspace);

      _bus.Invoke(new RedrawContainersCommand());
      _bus.Invoke(new FocusWindowCommand(focusedWindow));

      return CommandResponse.Ok;
    }
  }
}
