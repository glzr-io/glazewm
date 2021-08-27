using LarsWM.Infrastructure.Bussing;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.Workspaces.Commands;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Domain.Windows;
using System.Linq;
using System.Diagnostics;
using LarsWM.Domain.Containers;
using LarsWM.Domain.Containers.Commands;

namespace LarsWM.Domain.Workspaces.CommandHandlers
{
  class FocusWorkspaceHandler : ICommandHandler<FocusWorkspaceCommand>
  {
    private Bus _bus;
    private WorkspaceService _workspaceService;
    private MonitorService _monitorService;

    public FocusWorkspaceHandler(Bus bus, WorkspaceService workspaceService, MonitorService monitorService)
    {
      _bus = bus;
      _workspaceService = workspaceService;
      _monitorService = monitorService;
    }

    public dynamic Handle(FocusWorkspaceCommand command)
    {
      var workspaceName = command.WorkspaceName;

      // Get workspace to focus. If it's currently inactive, then activate it.
      var workspaceToFocus = _workspaceService.GetActiveWorkspaceByName(workspaceName)
        ?? ActivateWorkspace(workspaceName);

      // Get the currently focused workspace.
      var focusedWorkspace = _workspaceService.GetFocusedWorkspace();

      if (workspaceToFocus == focusedWorkspace)
        return CommandResponse.Ok;

      // Whether the focused workspace is the only workspace on the monitor.
      var isOnlyWorkspace = focusedWorkspace.Parent.Children.Count() == 1
        && workspaceToFocus.Parent != focusedWorkspace.Parent;

      // Destroy the currently focused workspace if it's empty.
      // TODO: Avoid destroying the workspace if `Workspace.KeepAlive` is enabled.
      if (!focusedWorkspace.HasChildren() && !isOnlyWorkspace)
        _bus.Invoke(new DetachWorkspaceFromMonitorCommand(focusedWorkspace));

      // Display the containers of the workspace to switch focus to.
      _bus.Invoke(new DisplayWorkspaceCommand(workspaceToFocus));

      // If workspace has no descendant windows, set focus to the workspace itself.
      if (!workspaceToFocus.HasChildren())
      {
        // Create a focus stack pointing to the workspace.
        _bus.Invoke(new CreateFocusStackCommand(workspaceToFocus));

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
    /// Activates a given workspace on the currently focused monitor.
    /// </summary>
    /// <param name="workspaceName">Name of the workspace to activate.</param>
    private Workspace ActivateWorkspace(string workspaceName)
    {
      var inactiveWorkspace = _workspaceService.InactiveWorkspaces.FirstOrDefault(workspace => workspace.Name == workspaceName);

      if (inactiveWorkspace == null)
      {
        // TODO: Handling this error is avoidable by checking that all focus commands in user config are valid.
        Debug.WriteLine($"Failed to activate workspace {workspaceName}. No such workspace exists.");
        return null;
      }

      var focusedMonitor = _monitorService.GetFocusedMonitor();
      _bus.Invoke(new AttachWorkspaceToMonitorCommand(inactiveWorkspace, focusedMonitor));

      return inactiveWorkspace;
    }
  }
}
