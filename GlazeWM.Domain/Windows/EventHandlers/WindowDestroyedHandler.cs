using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi.Events;

namespace GlazeWM.Domain.Windows.EventHandlers
{
  class WindowDestroyedHandler : IEventHandler<WindowDestroyedEvent>
  {
    private readonly Bus _bus;
    private readonly WindowService _windowService;

    public WindowDestroyedHandler(Bus bus, WindowService windowService)
    {
      _bus = bus;
      _windowService = windowService;
    }

    public void Handle(WindowDestroyedEvent @event)
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

      if (window == null)
        return;

      // If window is in tree, detach the removed window from its parent.
      _bus.Invoke(new RemoveWindowCommand(window));
      _bus.Invoke(new RedrawContainersCommand());
    }
  }
}
