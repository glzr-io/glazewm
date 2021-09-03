using System.Collections.Generic;
using System.Linq;
using System.Windows.Forms;
using LarsWM.Domain.Common.Enums;
using LarsWM.Domain.Containers;
using LarsWM.Domain.Windows;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Domain.Monitors
{
  public class MonitorService
  {
    private ContainerService _containerService;
    private WindowService _windowService;

    public MonitorService(ContainerService containerService, WindowService windowService)
    {
      _containerService = containerService;
      _windowService = windowService;
    }

    /// <summary>
    /// Get the root level of trees in container forest.
    /// </summary>
    public IEnumerable<Monitor> GetMonitors()
    {
      return _containerService.ContainerTree.Cast<Monitor>();
    }

    public Monitor GetMonitorFromChildContainer(Container container)
    {
      return container.TraverseUpEnumeration().OfType<Monitor>().First();
    }

    // TODO: If unreliable, consider using absolute X & Y of window and comparing it with X & Y of monitors.
    public Monitor GetMonitorFromUnaddedWindow(Window window)
    {
      var screen = Screen.FromHandle(window.Hwnd);

      var matchedMonitor = GetMonitors().FirstOrDefault(m => m.Screen.DeviceName == screen.DeviceName);

      if (matchedMonitor == null)
        return GetMonitors().First();

      return matchedMonitor;
    }

    public Monitor GetFocusedMonitor()
    {
      var focusedContainer = _containerService.FocusedContainer;
      return GetMonitorFromChildContainer(focusedContainer);
    }

    public uint GetMonitorDpi(Screen screen)
    {
      // Get a handle to the monitor from a `Screen`.
      var point = new System.Drawing.Point(screen.Bounds.Left + 1, screen.Bounds.Top + 1);
      var monitorHandle = MonitorFromPoint(point, MonitorFromPointFlags.MONITOR_DEFAULTTONEAREST);

      uint dpiX, dpiY;
      GetDpiForMonitor(monitorHandle, DpiType.Effective, out dpiX, out dpiY);

      return dpiX;
    }

    /// <summary>
    /// Get monitor in a given direction. Use i3wm's algorithm for finding best guess.
    /// </summary>
    /// <param name="direction">Direction to search in.</param>
    /// <param name="originMonitor">The monitor to search from.</param>
    /// <returns></returns>
    public Monitor GetMonitorInDirection(Direction direction, Monitor originMonitor)
    {
      Monitor monitorInDirection = null;

      foreach (var monitor in GetMonitors())
      {
        // Check whether the monitor is to the right/left of the origin monitor.
        if (
          (direction == Direction.RIGHT && monitor.X > originMonitor.X) ||
          (direction == Direction.LEFT && monitor.X < originMonitor.X)
        )
        {
          // Check whether the y-coordinate overlaps with the y-coordinate of the origin monitor.
          if (
            monitor.Y + monitor.Height <= originMonitor.Y ||
            originMonitor.Y + originMonitor.Height <= monitor.Y
          )
            continue;
        }
        // Check whether the monitor is below/above the origin monitor.
        else if (
          (direction == Direction.DOWN && monitor.Y > originMonitor.Y) ||
          (direction == Direction.UP && monitor.Y < originMonitor.Y)
        )
        {
          // Check whether the x-coordinate overlaps with the x-coordinate of the origin monitor.
          if (
            monitor.X + monitor.Width <= originMonitor.X ||
            originMonitor.X + originMonitor.Width <= monitor.X
          )
            continue;
        }
        else
          continue;

        // Initialize `monitorInDirection` if no other suitable monitors have been found yet.
        if (monitorInDirection == null)
        {
          monitorInDirection = monitor;
          continue;
        }

        // Reassign `monitorInDirection` if the monitor is closer.
        if ((direction == Direction.RIGHT && monitor.X < monitorInDirection.X) ||
            (direction == Direction.LEFT && monitor.X > monitorInDirection.X) ||
            (direction == Direction.DOWN && monitor.Y < monitorInDirection.Y) ||
            (direction == Direction.UP && monitor.Y > monitorInDirection.Y))
        {
          monitorInDirection = monitor;
        }
      }

      return monitorInDirection;
    }
  }
}
