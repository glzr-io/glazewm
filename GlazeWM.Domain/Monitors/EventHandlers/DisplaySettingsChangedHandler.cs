using System.Linq;
using System.Windows.Forms;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors.Commands;
using GlazeWM.Domain.Monitors.Events;
using GlazeWM.Domain.Windows;
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
    private WindowService _windowService;

    public DisplaySettingsChangedHandler(
      Bus bus,
      MonitorService monitorService,
      ContainerService containerService,
      WindowService windowService
    )
    {
      _bus = bus;
      _monitorService = monitorService;
      _containerService = containerService;
      _windowService = windowService;
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

      // Display setting changes can spread windows out sporadically, so mark all windows as needing
      // a DPI adjustment (just in case).
      foreach (var window in _windowService.GetWindows())
        window.HasPendingDpiAdjustment = true;

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
    }
  }
}
