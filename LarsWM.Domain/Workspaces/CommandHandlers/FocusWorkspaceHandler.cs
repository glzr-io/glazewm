using LarsWM.Infrastructure.Bussing;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.Workspaces.Commands;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Domain.Windows;
using System.Linq;
using System.Diagnostics;

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
      var workspaceToFocus = _workspaceService.GetActiveWorkspaceByName(workspaceName);

      if (workspaceToFocus == null)
      {
        var activatedWorkspace = ActivateWorkspace(workspaceName);
        workspaceToFocus = activatedWorkspace;
      }

      _bus.Invoke(new DisplayWorkspaceCommand(workspaceToFocus));

      // Set focus to the last focused window in workspace.
      if (workspaceToFocus.LastFocusedContainer != null)
        _bus.Invoke(new FocusWindowCommand(workspaceToFocus.LastFocusedContainer as Window));

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
