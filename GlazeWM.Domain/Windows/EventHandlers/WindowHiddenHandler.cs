using System.Linq;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Common.Utils;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Events;
using Microsoft.Extensions.Logging;

namespace GlazeWM.Domain.Windows.EventHandlers
{
  internal sealed class WindowHiddenHandler : IEventHandler<WindowHiddenEvent>
  {
    private readonly Bus _bus;
    private readonly WindowService _windowService;
    private readonly ILogger<WindowHiddenHandler> _logger;

    public WindowHiddenHandler(
      Bus bus,
      WindowService windowService,
      ILogger<WindowHiddenHandler> logger)
    {
      _bus = bus;
      _windowService = windowService;
      _logger = logger;
    }

    public void Handle(WindowHiddenEvent @event)
    {
      var windowHandle = @event.WindowHandle;

      if (_windowService.AppBarHandles.Contains(windowHandle))
      {
        _windowService.AppBarHandles.Remove(windowHandle);
        _bus.Invoke(new RefreshMonitorStateCommand());
        return;
      }

      var window = _windowService.GetWindows()
        .FirstOrDefault(window => window.Handle == windowHandle);

      // Ignore event if window is unmanaged.
      if (window is null)
        return;

      _logger.LogWindowEvent("Window hidden", window);

      // Update display state.
      if (window.DisplayState is DisplayState.Hiding)
      {
        window.DisplayState = DisplayState.Hidden;
        return;
      }

      // Detach the hidden window from its parent.
      if (window.DisplayState is DisplayState.Shown && !WindowService.IsHandleVisible(window.Handle))
      {
        _bus.Invoke(new UnmanageWindowCommand(window));
        _bus.Invoke(new RedrawContainersCommand());
        _bus.Invoke(new SyncNativeFocusCommand());
      }
    }
  }
}

// Scenario 1:
// window hiding (due to focus defocus)
// window showing (due to focus refocus)
// window hidden event
// window shown event

// Scenario 2:
// window hiding (due to focus defocus)
// window hidden event
// window showing (due to focus refocus)
// window shown event

// Scenario 3:
// window hiding (due to focus defocus)
// window showing (due to focus refocus)
// window hiding (due to focus defocus)
// window hidden event
// window shown event
// window hidden event
