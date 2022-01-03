using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors.Commands;
using GlazeWM.Domain.Monitors.Events;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Monitors.CommandHandlers
{
  class RemoveMonitorHandler : ICommandHandler<RemoveMonitorCommand>
  {
    private Bus _bus;
    private MonitorService _monitorService;

    public RemoveMonitorHandler(Bus bus, MonitorService monitorService)
    {
      _bus = bus;
      _monitorService = monitorService;
    }

    public CommandResponse Handle(RemoveMonitorCommand command)
    {
      var monitorToRemove = command.MonitorToRemove;
      var targetMonitor = _monitorService.GetMonitors().First(
        monitor => monitor != monitorToRemove
      );

      // Keep reference to the focused monitor prior to moving workspaces around.
      var focusedMonitor = _monitorService.GetFocusedMonitor();

      // Avoid moving empty workspaces.
      var workspacesToMove = monitorToRemove.Children.Where(workspace => workspace.HasChildren());

      foreach (var workspace in workspacesToMove.ToList())
      {
        // Move workspace to target monitor.
        _bus.Invoke(new MoveContainerWithinTreeCommand(workspace, targetMonitor, false));

        // Get windows of the moved workspace.
        var windows = workspace.Descendants
          .Where(descendant => descendant is Window)
          .Cast<Window>();

        // Adjust floating position of moved windows.
        // TODO: If primary monitor changes, does floating placement of all windows need to be updated?
        foreach (var window in windows)
          window.FloatingPlacement =
            window.FloatingPlacement.TranslateToCenter(workspace.ToRectangle());
      }

      _bus.Invoke(new DetachContainerCommand(monitorToRemove));
      _bus.RaiseEvent(new MonitorRemovedEvent(monitorToRemove.DeviceName));

      if (focusedMonitor == monitorToRemove)
        _bus.Invoke(new FocusWorkspaceCommand(targetMonitor.DisplayedWorkspace.Name));

      return CommandResponse.Ok;
    }
  }
}
