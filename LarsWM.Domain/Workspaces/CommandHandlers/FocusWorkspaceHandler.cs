using LarsWM.Infrastructure.Bussing;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.Workspaces.Commands;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Domain.Windows;
using System.Linq;
using System.Diagnostics;
using LarsWM.Domain.Containers;
using LarsWM.Domain.Containers.Events;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Domain.Workspaces.CommandHandlers
{
  class FocusWorkspaceHandler : ICommandHandler<FocusWorkspaceCommand>
  {
    private Bus _bus;
    private WorkspaceService _workspaceService;
    private MonitorService _monitorService;
    private ContainerService _containerService;

    public FocusWorkspaceHandler(Bus bus, WorkspaceService workspaceService, MonitorService monitorService, ContainerService containerService)
    {
      _bus = bus;
      _workspaceService = workspaceService;
      _monitorService = monitorService;
      _containerService = containerService;
    }

    public CommandResponse Handle(FocusWorkspaceCommand command)
    {
      var workspaceName = command.WorkspaceName;

      // Get workspace to focus. If it's currently inactive, then activate it.
      var workspaceToFocus = _workspaceService.GetActiveWorkspaceByName(workspaceName)
        ?? ActivateWorkspace(workspaceName);

      // Get the currently focused workspace. This can be null if there currently
      // isn't a container that has focus.
      var focusedWorkspace = _workspaceService.GetFocusedWorkspace();

      if (workspaceToFocus == focusedWorkspace)
        return CommandResponse.Ok;

      // Whether the focused workspace is the only workspace on the monitor.
      var isOnlyWorkspace = focusedWorkspace?.Parent?.Children?.Count() == 1
        && workspaceToFocus.Parent != focusedWorkspace.Parent;

      // Destroy the currently focused workspace if it's empty.
      // TODO: Avoid destroying the workspace if `Workspace.KeepAlive` is enabled.
      if (focusedWorkspace != null && !focusedWorkspace.HasChildren() && !isOnlyWorkspace)
        _bus.Invoke(new DetachWorkspaceFromMonitorCommand(focusedWorkspace));

      // Display the containers of the workspace to switch focus to.
      _bus.Invoke(new DisplayWorkspaceCommand(workspaceToFocus));

      // If workspace has no descendant windows, set focus to the workspace itself.
      if (!workspaceToFocus.HasChildren())
      {
        _bus.RaiseEvent(new FocusChangedEvent(workspaceToFocus));

        // Remove focus from whichever window currently has focus.
        SetForegroundWindow(GetDesktopWindow());

        return CommandResponse.Ok;
      }

      // Set focus to the last focused window in workspace (if there is one).
      if (workspaceToFocus.LastFocusedDescendant != null)
      {
        _bus.Invoke(new FocusWindowCommand(workspaceToFocus.LastFocusedDescendant as Window));
        return CommandResponse.Ok;
      }

      // Set focus to an arbitrary window.
      var arbitraryWindow = workspaceToFocus.Flatten().OfType<Window>().First();
      _bus.Invoke(new FocusWindowCommand(arbitraryWindow));

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
