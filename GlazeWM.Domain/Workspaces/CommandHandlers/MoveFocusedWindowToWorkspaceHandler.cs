using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Containers.Events;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
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

      var currentWorkspace = _workspaceService.GetWorkspaceFromChildContainer(focusedWindow);
      var targetWorkspace = _workspaceService.GetActiveWorkspaceByName(workspaceName)
        ?? ActivateWorkspace(workspaceName);

      var insertionTarget = targetWorkspace.LastFocusedDescendant;

      if (insertionTarget == null)
        _bus.Invoke(new AttachContainerCommand(targetWorkspace, focusedWindow));
      else
        _bus.Invoke(new AttachContainerCommand(insertionTarget.Parent as SplitContainer, focusedWindow, insertionTarget.Index + 1));

      _containerService.SplitContainersToRedraw.Add(currentWorkspace);

      // Whether the current workspace is the only workspace on the monitor.
      var isOnlyWorkspace = currentWorkspace.Parent.Children.Count == 1
        && targetWorkspace.Parent != currentWorkspace.Parent;

      // Destroy the current workspace if it's empty.
      // TODO: Avoid destroying the workspace if `Workspace.KeepAlive` is enabled.
      if (currentWorkspace != null && !currentWorkspace.HasChildren() && !isOnlyWorkspace)
        _bus.Invoke(new DetachWorkspaceFromMonitorCommand(currentWorkspace));

      // Display the containers of the workspace to switch focus to.
      _bus.Invoke(new DisplayWorkspaceCommand(targetWorkspace));
      _bus.Invoke(new RedrawContainersCommand());
      _bus.RaiseEvent(new FocusChangedEvent(focusedWindow));

      return CommandResponse.Ok;
    }

    /// <summary>
    /// Activate a given workspace on the currently focused monitor.
    /// </summary>
    /// <param name="workspaceName">Name of the workspace to activate.</param>
    private Workspace ActivateWorkspace(string workspaceName)
    {
      var inactiveWorkspace = _workspaceService.GetInactiveWorkspaceByName(workspaceName);
      var focusedMonitor = _monitorService.GetFocusedMonitor();

      _bus.Invoke(new AttachWorkspaceToMonitorCommand(inactiveWorkspace, focusedMonitor));

      return inactiveWorkspace;
    }
  }
}
