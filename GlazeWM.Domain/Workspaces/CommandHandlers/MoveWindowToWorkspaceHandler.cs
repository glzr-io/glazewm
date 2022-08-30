using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows.Commands;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
{
  internal class MoveWindowToWorkspaceHandler : ICommandHandler<MoveWindowToWorkspaceCommand>
  {
    private readonly Bus _bus;
    private readonly WorkspaceService _workspaceService;
    private readonly ContainerService _containerService;
    private readonly MonitorService _monitorService;

    public MoveWindowToWorkspaceHandler(
      Bus bus,
      WorkspaceService workspaceService,
      ContainerService containerService,
      MonitorService monitorService
    )
    {
      _bus = bus;
      _workspaceService = workspaceService;
      _containerService = containerService;
      _monitorService = monitorService;
    }

    public CommandResponse Handle(MoveWindowToWorkspaceCommand command)
    {
      var windowToMove = command.WindowToMove;
      var workspaceName = command.WorkspaceName;

      var currentWorkspace = WorkspaceService.GetWorkspaceFromChildContainer(windowToMove);
      // Make sure workspace opens on the appropriate monitor
      var currentMonitor = _monitorService.GetMonitorForWorkspace(workspaceName) ?? MonitorService.GetMonitorFromChildContainer(currentWorkspace);

      var targetWorkspace = _workspaceService.GetActiveWorkspaceByName(workspaceName)
        ?? ActivateWorkspace(workspaceName, currentMonitor);

      // Since target workspace could be on a different monitor, adjustments might need to be made
      // because of DPI.
      if (MonitorService.HasDpiDifference(currentWorkspace, targetWorkspace))
        windowToMove.HasPendingDpiAdjustment = true;

      // Update floating placement if the window has to cross monitors.
      if (targetWorkspace.Parent != currentWorkspace.Parent)
        windowToMove.FloatingPlacement =
          windowToMove.FloatingPlacement.TranslateToCenter(targetWorkspace.ToRectangle());

      if (windowToMove is TilingWindow)
        MoveTilingWindowToWorkspace(windowToMove as TilingWindow, targetWorkspace);
      else
        _bus.Invoke(new MoveContainerWithinTreeCommand(windowToMove, targetWorkspace, false));

      // Reassign focus to descendant within the current workspace.
      ReassignFocusWithinWorkspace(currentWorkspace);

      _containerService.ContainersToRedraw.Add(currentWorkspace);
      _containerService.ContainersToRedraw.Add(windowToMove);
      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }

    private Workspace ActivateWorkspace(string workspaceName, Monitor targetMonitor)
    {
      _bus.Invoke(new ActivateWorkspaceCommand(workspaceName, targetMonitor));
      return _workspaceService.GetActiveWorkspaceByName(workspaceName);
    }

    private void MoveTilingWindowToWorkspace(TilingWindow windowToMove, Workspace targetWorkspace)
    {
      var insertionTarget = targetWorkspace.LastFocusedDescendantOfType(typeof(IResizable));

      // Insert the window into the target workspace.
      if (insertionTarget == null)
        _bus.Invoke(new MoveContainerWithinTreeCommand(windowToMove, targetWorkspace, true));
      else
        _bus.Invoke(
          new MoveContainerWithinTreeCommand(
            windowToMove,
            insertionTarget.Parent,
            insertionTarget.Index + 1,
            true
          )
        );
    }

    private void ReassignFocusWithinWorkspace(Workspace workspace)
    {
      var containerToFocus = workspace.LastFocusedDescendant ?? workspace;
      _bus.Invoke(new SetFocusedDescendantCommand(containerToFocus));

      if (containerToFocus is Window)
        _bus.Invoke(new FocusWindowCommand(containerToFocus as Window));
      else
      {
        KeybdEvent(0, 0, 0, 0);
        SetForegroundWindow(GetDesktopWindow());
      }
    }
  }
}
