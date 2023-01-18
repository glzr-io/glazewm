using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
{
  internal sealed class MoveWindowToWorkspaceHandler : ICommandHandler<MoveWindowToWorkspaceCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;
    private readonly MonitorService _monitorService;
    private readonly UserConfigService _userConfigService;
    private readonly WorkspaceService _workspaceService;

    public MoveWindowToWorkspaceHandler(
      Bus bus,
      ContainerService containerService,
      MonitorService monitorService,
      UserConfigService userConfigService,
      WorkspaceService workspaceService)
    {
      _bus = bus;
      _containerService = containerService;
      _monitorService = monitorService;
      _userConfigService = userConfigService;
      _workspaceService = workspaceService;
    }

    public CommandResponse Handle(MoveWindowToWorkspaceCommand command)
    {
      var windowToMove = command.WindowToMove;
      var workspaceName = command.WorkspaceName;

      var currentWorkspace = WorkspaceService.GetWorkspaceFromChildContainer(windowToMove);
      var targetWorkspace = _workspaceService.GetActiveWorkspaceByName(workspaceName)
        ?? ActivateWorkspace(workspaceName, windowToMove);

      if (currentWorkspace == targetWorkspace)
        return CommandResponse.Ok;

      // Since target workspace could be on a different monitor, adjustments might need to be made
      // because of DPI.
      if (MonitorService.HasDpiDifference(currentWorkspace, targetWorkspace))
        windowToMove.HasPendingDpiAdjustment = true;

      // Update floating placement if the window has to cross monitors.
      if (targetWorkspace.Parent != currentWorkspace.Parent)
        windowToMove.FloatingPlacement =
          windowToMove.FloatingPlacement.TranslateToCenter(targetWorkspace.ToRect());

      var focusTarget = WindowService.GetFocusTargetAfterRemoval(windowToMove);

      // Since the workspace that gets displayed is the last focused child, focus needs to be
      // reassigned to the displayed workspace.
      var targetMonitor = targetWorkspace.Parent as Monitor;
      var focusResetTarget = targetWorkspace.IsDisplayed ? null : targetMonitor.LastFocusedDescendant;

      if (windowToMove is TilingWindow)
        MoveTilingWindowToWorkspace(windowToMove as TilingWindow, targetWorkspace);
      else
        _bus.Invoke(new MoveContainerWithinTreeCommand(windowToMove, targetWorkspace, false));

      if (focusResetTarget is not null)
        _bus.Invoke(new SetFocusedDescendantCommand(focusResetTarget));

      // Reassign focus to descendant within the current workspace. Need to call
      // `SetFocusedDescendantCommand` for when commands like `FocusWorkspaceCommand` are called
      // immediately afterwards and they should behave as if `focusTarget` is the focused
      // descendant.
      _bus.Invoke(new SetFocusedDescendantCommand(focusTarget));
      _bus.Invoke(new SetNativeFocusCommand(focusTarget));

      _containerService.ContainersToRedraw.Add(currentWorkspace);
      _containerService.ContainersToRedraw.Add(windowToMove);
      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }

    private Workspace ActivateWorkspace(string workspaceName, Window windowToMove)
    {
      var currentMonitor = MonitorService.GetMonitorFromChildContainer(windowToMove);

      // Get the monitor that the workspace should be bound to (if it exists).
      var workspaceConfig = _userConfigService.GetWorkspaceConfigByName(workspaceName);
      var boundMonitor =
        _monitorService.GetMonitorByDeviceName(workspaceConfig.BindToMonitor);

      // Activate the workspace on the target monitor.
      var targetMonitor = boundMonitor ?? currentMonitor;
      _bus.Invoke(new ActivateWorkspaceCommand(workspaceName, targetMonitor));

      return _workspaceService.GetActiveWorkspaceByName(workspaceName);
    }

    private void MoveTilingWindowToWorkspace(TilingWindow windowToMove, Workspace targetWorkspace)
    {
      var insertionTarget = targetWorkspace.LastFocusedDescendantOfType<IResizable>();

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
  }
}
