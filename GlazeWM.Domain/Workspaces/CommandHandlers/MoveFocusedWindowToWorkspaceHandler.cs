using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Containers.Commands;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
{
  class MoveFocusedWindowToWorkspaceHandler : ICommandHandler<MoveFocusedWindowToWorkspaceCommand>
  {
    private readonly Bus _bus;
    private readonly WorkspaceService _workspaceService;
    private readonly MonitorService _monitorService;
    private readonly ContainerService _containerService;

    public MoveFocusedWindowToWorkspaceHandler(
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

    public CommandResponse Handle(MoveFocusedWindowToWorkspaceCommand command)
    {
      var workspaceName = command.WorkspaceName;
      var focusedWindow = _containerService.FocusedContainer as Window;

      // Ignore cases where focused container is not a window or not in foreground.
      if (focusedWindow == null || !_containerService.IsFocusSynced)
        return CommandResponse.Ok;

      var currentWorkspace = _workspaceService.GetFocusedWorkspace();
      var targetWorkspace = _workspaceService.GetActiveWorkspaceByName(workspaceName)
        ?? ActivateWorkspace(workspaceName);

      // Since target workspace could be on a different monitor, adjustments might need to be made
      // because of DPI.
      if (MonitorService.HasDpiDifference(currentWorkspace, targetWorkspace))
        focusedWindow.HasPendingDpiAdjustment = true;

      // Update floating placement if the window has to cross monitors.
      if (targetWorkspace.Parent != currentWorkspace.Parent)
        focusedWindow.FloatingPlacement =
          focusedWindow.FloatingPlacement.TranslateToCenter(targetWorkspace.ToRectangle());

      if (focusedWindow is TilingWindow)
        MoveTilingWindowToWorkspace(focusedWindow as TilingWindow, targetWorkspace);
      else
        _bus.Invoke(new MoveContainerWithinTreeCommand(focusedWindow, targetWorkspace, false));

      // Reassign focus to descendant within the current workspace.
      _bus.Invoke(new FocusWorkspaceCommand(currentWorkspace.Name));

      _containerService.ContainersToRedraw.Add(currentWorkspace);
      _containerService.ContainersToRedraw.Add(targetWorkspace);
      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }

    private void MoveTilingWindowToWorkspace(TilingWindow focusedWindow, Workspace targetWorkspace)
    {
      var insertionTarget = targetWorkspace.LastFocusedDescendantOfType(typeof(IResizable));

      // Insert the focused window into the target workspace.
      if (insertionTarget == null)
        _bus.Invoke(new MoveContainerWithinTreeCommand(focusedWindow, targetWorkspace, true));
      else
        _bus.Invoke(
          new MoveContainerWithinTreeCommand(
            focusedWindow,
            insertionTarget.Parent,
            insertionTarget.Index + 1,
            true
          )
        );
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
