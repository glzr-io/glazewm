using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces.Commands;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
{
  internal class MoveWindowToWorkspaceHandler : ICommandHandler<MoveWindowToWorkspaceCommand>
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

      // Since target workspace could be on a different monitor, adjustments might need to be made
      // because of DPI.
      if (MonitorService.HasDpiDifference(currentWorkspace, targetWorkspace))
        windowToMove.HasPendingDpiAdjustment = true;

      // Update floating placement if the window has to cross monitors.
      if (targetWorkspace.Parent != currentWorkspace.Parent)
        windowToMove.FloatingPlacement =
          windowToMove.FloatingPlacement.TranslateToCenter(targetWorkspace.ToRect());

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
