using System.Linq;
using System.Windows.Forms;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors.Commands;
using GlazeWM.Domain.Monitors.Events;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi.Events;

namespace GlazeWM.Domain.Monitors.EventHandlers
{
  class DisplaySettingsChangedHandler : IEventHandler<DisplaySettingsChangedEvent>
  {
    private Bus _bus;
    private MonitorService _monitorService;
    private ContainerService _containerService;

    public DisplaySettingsChangedHandler(Bus bus, MonitorService monitorService, ContainerService containerService)
    {
      _bus = bus;
      _monitorService = monitorService;
      _containerService = containerService;
    }

    public void Handle(DisplaySettingsChangedEvent @event)
    {
      foreach (var screen in Screen.AllScreens)
      {
        var foundMonitor = _monitorService.GetMonitors().FirstOrDefault(
          monitor => monitor.DeviceName == screen.DeviceName
        );

        // Add monitor if it doesn't exist in state.
        if (foundMonitor == null)
        {
          _bus.Invoke(new AddMonitorCommand(screen));
          continue;
        }

        // Update monitor with changes to dimensions and positioning.
        foundMonitor.Width = screen.WorkingArea.Width;
        foundMonitor.Height = screen.WorkingArea.Height;
        foundMonitor.X = screen.WorkingArea.X;
        foundMonitor.Y = screen.WorkingArea.Y;
        foundMonitor.IsPrimary = screen.Primary;
      }

      // Verify that all monitors in state exist in `Screen.AllScreens`.
      var monitorsToRemove = _monitorService.GetMonitors().Where(
        monitor => !IsMonitorActive(monitor)
      );

      // Remove any monitors that no longer exist and move their workspaces to other monitors.
      // TODO: Verify that this works with "Duplicate these displays" or "Show only X" settings.
      foreach (var monitor in monitorsToRemove.ToList())
        RemoveMonitor(monitor);

      // Redraw full container tree.
      _containerService.ContainersToRedraw.Add(_containerService.ContainerTree);
      _bus.Invoke(new RedrawContainersCommand());
    }

    private bool IsMonitorActive(Monitor monitor)
    {
      return Screen.AllScreens.Any(screen => screen.DeviceName == monitor.DeviceName);
    }

    // TODO: Move to own command.
    private void RemoveMonitor(Monitor monitorToRemove)
    {
      // Keep reference to the focused monitor prior to moving workspaces around.
      var focusedMonitor = _monitorService.GetFocusedMonitor();

      var targetMonitor = _monitorService.GetMonitors().First(
        monitor => monitor != monitorToRemove
      );

      // Avoid moving empty workspaces.
      var workspacesToMove = monitorToRemove.Children.Where(workspace => workspace.HasChildren());

      foreach (var workspace in workspacesToMove.ToList())
        _bus.Invoke(new MoveContainerWithinTreeCommand(workspace, targetMonitor, false));

      // TODO: Mark windows as needing DPI adjustment if needed.
      // TODO: Adjust floating position of moved windows.

      _bus.Invoke(new DetachContainerCommand(monitorToRemove));
      _bus.RaiseEvent(new MonitorRemovedEvent(monitorToRemove.DeviceName));

      if (focusedMonitor == monitorToRemove)
        _bus.Invoke(new FocusWorkspaceCommand(targetMonitor.DisplayedWorkspace.Name));
    }
  }
}
