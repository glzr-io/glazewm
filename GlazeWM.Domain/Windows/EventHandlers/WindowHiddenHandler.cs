using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi.Events;
using Microsoft.Extensions.Logging;

namespace GlazeWM.Domain.Windows.EventHandlers
{
  internal class WindowHiddenHandler : IEventHandler<WindowHiddenEvent>
  {
    private readonly Bus _bus;
    private readonly WindowService _windowService;
    private readonly ILogger<WindowHiddenHandler> _logger;

    public WindowHiddenHandler(
      Bus bus,
      WindowService windowService,
      ILogger<WindowHiddenHandler> logger
    )
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
        .FirstOrDefault(window => window.Hwnd == windowHandle);

      // Ignore events where the window isn't managed or is actually supposed to be hidden. Since
      // window events are processed in a sequence, also handle case where the window is not
      // actually hidden anymore when the event is processed.
      if (window?.IsHidden != false || WindowService.IsHandleVisible(window.Hwnd))
        return;

      _logger.LogDebug($"Window hidden {window.ProcessName} | {window.ClassName}");

      // Detach the hidden window from its parent.
      _bus.Invoke(new RemoveWindowCommand(window));
      _bus.Invoke(new RedrawContainersCommand());
    }
  }
}
