using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.Containers.Commands;
using System.Linq;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
{
  internal class FocusWorkspaceHandler : ICommandHandler<FocusWorkspaceCommand>
  {
    private readonly Bus _bus;
    private readonly WorkspaceService _workspaceService;
    private readonly MonitorService _monitorService;
    private readonly ContainerService _containerService;

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

      if (focusedWorkspace == workspaceToFocus)
        return CommandResponse.Ok;

      // Set focus to the last focused window in workspace. If the workspace has no descendant
      // windows, then set focus to the workspace itself.
      var containerToFocus = workspaceToFocus.HasChildren()
        ? workspaceToFocus.LastFocusedDescendant
        : workspaceToFocus;

      _bus.Invoke(new SetFocusedDescendantCommand(containerToFocus));
      _bus.RaiseEvent(new FocusChangedEvent(containerToFocus));

      // Display the workspace to switch focus to.
      _containerService.ContainersToRedraw.Add(focusedWorkspace);
      _containerService.ContainersToRedraw.Add(workspaceToFocus);
      _bus.Invoke(new RedrawContainersCommand());

      // Container to focus is either a window or a workspace.
      if (containerToFocus is Window)
        _bus.Invoke(new FocusWindowCommand(containerToFocus as Window));
      else
      {
        // Remove focus from whichever window currently has focus.
        KeybdEvent(0, 0, 0, 0);
        SetForegroundWindow(GetDesktopWindow());
      }

      // Get empty workspace to destroy (if any are found). Cannot destroy empty workspaces if
      // they're the only workspace on the monitor or are pending focus.
      var workspaceToDestroy = _workspaceService.GetActiveWorkspaces()
        .FirstOrDefault(workspace =>
        {
          return !workspace.KeepAlive
            &&!workspace.HasChildren()
            && !workspace.IsDisplayed
            && _containerService.PendingFocusContainer != workspace;
        });

      if (workspaceToDestroy != null)
        _bus.Invoke(new DeactivateWorkspaceCommand(workspaceToDestroy));

      return CommandResponse.Ok;
    }

    /// <summary>
    /// Activate a given workspace on the currently focused monitor.
    /// </summary>
    private Workspace ActivateWorkspace(string workspaceName)
    {
      var targetMonitor = _monitorService.GetMonitorForWorkspace(workspaceName) ?? _monitorService.GetFocusedMonitor();
      _bus.Invoke(new ActivateWorkspaceCommand(workspaceName, targetMonitor));

      return _workspaceService.GetActiveWorkspaceByName(workspaceName);
    }
  }
}
