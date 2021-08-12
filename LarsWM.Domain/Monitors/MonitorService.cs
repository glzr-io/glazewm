using System;
using System.Collections.Generic;
using System.Linq;
using System.Windows.Forms;
using LarsWM.Domain.Containers;
using LarsWM.Domain.Windows;

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
      var focusedWindow = _windowService.FocusedWindow;
      return GetMonitorFromChildContainer(focusedWindow);
    }
  }
}
