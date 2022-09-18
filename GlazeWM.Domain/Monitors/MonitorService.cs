using System;
using System.Collections.Generic;
using System.Linq;
using System.Windows.Forms;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers;
using GlazeWM.Infrastructure.WindowsApi;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Monitors
{
  public class MonitorService
  {
    private readonly ContainerService _containerService;

    public MonitorService(ContainerService containerService)
    {
      _containerService = containerService;
    }

    /// <summary>
    /// Get monitors by iterating over the children of the root container.
    /// </summary>
    public IEnumerable<Monitor> GetMonitors()
    {
      return _containerService.ContainerTree.Children.Cast<Monitor>();
    }

    public Monitor GetMonitorByDeviceName(string deviceName)
    {
      return GetMonitors().FirstOrDefault(monitor => monitor.DeviceName == deviceName);
    }

    public static Monitor GetMonitorFromChildContainer(Container container)
    {
      return container.SelfAndAncestors.OfType<Monitor>().First();
    }

    /// <summary>
    /// Get the monitor that encompasses the largest portion of the window handle.
    /// </summary>
    public Monitor GetMonitorFromHandleLocation(IntPtr windowHandle)
    {
      var screen = Screen.FromHandle(windowHandle);

      return GetMonitorByDeviceName(screen.DeviceName) ?? GetMonitors().First();
    }

    public Monitor GetFocusedMonitor()
    {
      return GetMonitorFromChildContainer(_containerService.FocusedContainer);
    }

    public static uint GetMonitorDpi(Monitor monitor)
    {
      // Create a point within the monitor's dimensions.
      var point = new Point
      {
        X = monitor.X + 1,
        Y = monitor.Y + 1
      };

      // Get a handle to the monitor.
      // TODO: Consider adding a `Monitor` getter for a monitor's handle.
      var monitorHandle = MonitorFromPoint(point, MonitorFromPointFlags.MONITOR_DEFAULTTONEAREST);
      _ = GetDpiForMonitor(monitorHandle, DpiType.Effective, out var dpiX, out _);

      // DPI X and Y should be equivalent, so it's arbitrary which to return.
      return dpiX;
    }

    /// <summary>
    /// Whether there is a difference in DPI between two containers.
    /// </summary>
    public static bool HasDpiDifference(Container firstContainer, Container secondContainer)
    {
      var firstMonitor = firstContainer is Monitor ?
        firstContainer as Monitor : GetMonitorFromChildContainer(firstContainer);

      var secondMonitor = secondContainer is Monitor ?
        secondContainer as Monitor : GetMonitorFromChildContainer(secondContainer);

      return firstMonitor.Dpi != secondMonitor.Dpi;
    }

    /// <summary>
    /// Get monitor in a given direction. Use i3wm's algorithm for finding best guess.
    /// </summary>
    /// <param name="direction">Direction to search in.</param>
    /// <param name="originMonitor">The monitor to search from.</param>
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
        // Skip monitors that are not in the given direction.
        else
          continue;

        // Set `monitorInDirection` if no suitable monitors have been found yet or if the monitor
        // is closer.
        if (
          monitorInDirection == null ||
          (direction == Direction.RIGHT && monitor.X < monitorInDirection.X) ||
          (direction == Direction.LEFT && monitor.X > monitorInDirection.X) ||
          (direction == Direction.DOWN && monitor.Y < monitorInDirection.Y) ||
          (direction == Direction.UP && monitor.Y > monitorInDirection.Y)
        )
          monitorInDirection = monitor;
      }

      return monitorInDirection;
    }
  }
}
