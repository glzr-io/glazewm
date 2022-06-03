using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Containers.Commands;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
{
  internal class MoveWindowToWorkspaceHandler : ICommandHandler<MoveWindowToWorkspaceCommand>
  {
    private readonly Bus _bus;
    private readonly WorkspaceService _workspaceService;
    private readonly ContainerService _containerService;

    public MoveWindowToWorkspaceHandler(
      Bus bus,
      WorkspaceService workspaceService,
      ContainerService containerService
    )
    {
      _bus = bus;
      _workspaceService = workspaceService;
      _containerService = containerService;
    }

    public CommandResponse Handle(MoveWindowToWorkspaceCommand command)
    {
      var windowToMove = command.WindowToMove;
      var workspaceName = command.WorkspaceName;

      var currentWorkspace = WorkspaceService.GetWorkspaceFromChildContainer(windowToMove);
      var currentMonitor = MonitorService.GetMonitorFromChildContainer(currentWorkspace);
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
      _bus.Invoke(new FocusWorkspaceCommand(currentWorkspace.Name));

      _containerService.ContainersToRedraw.Add(currentWorkspace);
      _containerService.ContainersToRedraw.Add(targetWorkspace);
      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
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

    private Workspace ActivateWorkspace(string workspaceName, Monitor targetMonitor)
    {
      _bus.Invoke(new ActivateWorkspaceCommand(workspaceName, targetMonitor));
      return _workspaceService.GetActiveWorkspaceByName(workspaceName);
    }
  }
}
