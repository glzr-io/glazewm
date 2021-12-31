using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.Containers.Commands;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;
using System.Linq;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
{
  class FocusWorkspaceHandler : ICommandHandler<FocusWorkspaceCommand>
  {
    private Bus _bus;
    private WorkspaceService _workspaceService;
    private MonitorService _monitorService;
    private ContainerService _containerService;

    public FocusWorkspaceHandler(
      Bus bus,
      WorkspaceService workspaceService,
      MonitorService monitorService,
      ContainerService containerService
    )
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

      // Display the containers of the workspace to switch focus to.
      _bus.Invoke(new DisplayWorkspaceCommand(workspaceToFocus));

      // Get empty workspace to destroy (if any are found). Cannot destroy empty workspaces if
      // they're the only workspace on the monitor or are pending focus.
      var workspaceToDestroy = _workspaceService.GetActiveWorkspaces()
        .FirstOrDefault(workspace =>
        {
          // TODO: Avoid destroying the workspace if `Workspace.KeepAlive` is enabled.
          return !workspace.HasChildren()
            && !workspace.IsDisplayed
            && _containerService.PendingFocusContainer != workspace;
        });

      if (workspaceToDestroy != null)
        _bus.Invoke(new DetachWorkspaceFromMonitorCommand(workspaceToDestroy));

      // If workspace has no descendant windows, set focus to the workspace itself.
      if (!workspaceToFocus.HasChildren())
      {
        _bus.Invoke(new SetFocusedDescendantCommand(workspaceToFocus));
        _bus.RaiseEvent(new FocusChangedEvent(workspaceToFocus));

        // Remove focus from whichever window currently has focus.
        KeybdEvent(0, 0, 0, 0);
        SetForegroundWindow(GetDesktopWindow());

        return CommandResponse.Ok;
      }

      // Set focus to the last focused window in workspace.
      _bus.Invoke(new FocusWindowCommand(workspaceToFocus.LastFocusedDescendant as Window));

      return CommandResponse.Ok;
    }

    /// <summary>
    /// Activate a given workspace on the currently focused monitor.
    /// </summary>
    private Workspace ActivateWorkspace(string workspaceName)
    {
      var focusedMonitor = _monitorService.GetFocusedMonitor();
      _bus.Invoke(new ActivateWorkspaceCommand(workspaceName, focusedMonitor));

      return _workspaceService.GetActiveWorkspaceByName(workspaceName);
    }
  }
}
