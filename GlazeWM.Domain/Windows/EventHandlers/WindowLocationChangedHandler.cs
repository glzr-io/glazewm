using GlazeWM.Domain.Monitors.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Events;

namespace GlazeWM.Domain.Windows.EventHandlers
{
  internal sealed class WindowLocationChangedHandler : IEventHandler<WindowLocationChangedEvent>
  {
    private readonly Bus _bus;
    private readonly WindowService _windowService;

    public WindowLocationChangedHandler(Bus bus, WindowService windowService)
    {
      _bus = bus;
      _windowService = windowService;
    }

    public void Handle(WindowLocationChangedEvent @event)
    {
      var windowHandle = @event.WindowHandle;

      if (!_windowService.AppBarHandles.Contains(windowHandle))
        return;

      _bus.Invoke(new RefreshMonitorStateCommand());
    }
  }
}
