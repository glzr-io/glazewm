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
    private IBus _bus;
    private WorkspaceService _workspaceService;
    private MonitorService _monitorService;

    public FocusWorkspaceHandler(IBus bus, WorkspaceService workspaceService, MonitorService monitorService)
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
        var inactiveWorkspace = _workspaceService.InactiveWorkspaces.FirstOrDefault(workspace => workspace.Name == workspaceName);

        if (inactiveWorkspace == null)
        {
          Debug.WriteLine($"Failed to focus on workspace {workspaceName}. No such workspace exists.");
          return null;
        }

        var focusedMonitor = _monitorService.GetFocusedMonitor();
        _bus.Invoke(new AttachWorkspaceToMonitorCommand(inactiveWorkspace, focusedMonitor));

        workspaceToFocus = inactiveWorkspace;
      }

      _bus.Invoke(new DisplayWorkspaceCommand(workspaceToFocus));

      // Set focus to the last focused window in workspace.
      if (workspaceToFocus.LastFocusedContainer != null)
        _bus.Invoke(new FocusWindowCommand(workspaceToFocus.LastFocusedContainer as Window));

      return CommandResponse.Ok;
    }
  }
}
