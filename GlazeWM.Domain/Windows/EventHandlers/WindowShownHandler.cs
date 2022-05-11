using System.Linq;
using GlazeWM.Domain.Monitors.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi.Events;

namespace GlazeWM.Domain.Windows.EventHandlers
{
  internal class WindowShownHandler : IEventHandler<WindowShownEvent>
  {
    private readonly Bus _bus;
    private readonly WindowService _windowService;

    public WindowShownHandler(Bus bus, WindowService windowService)
    {
      _bus = bus;
      _windowService = windowService;
    }

    public void Handle(WindowShownEvent @event)
    {
      var windowHandle = @event.WindowHandle;

      if (_windowService.IsHandleAppBar(windowHandle))
      {
        _windowService.AppBarHandles.Add(windowHandle);
        _bus.Invoke(new RefreshMonitorStateCommand());
        return;
      }

      var window = _windowService.GetWindows()
        .FirstOrDefault(window => window.Hwnd == windowHandle);

      // Ignore cases where window is already managed.
      if (window is not null || !WindowService.IsHandleManageable(windowHandle))
        return;

      _bus.Invoke(new AddWindowCommand(@event.WindowHandle));
    }
  }
}
