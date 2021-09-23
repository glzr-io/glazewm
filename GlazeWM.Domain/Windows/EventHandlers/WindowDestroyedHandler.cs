using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi.Events;

namespace GlazeWM.Domain.Windows.EventHandlers
{
  class WindowDestroyedHandler : IEventHandler<WindowDestroyedEvent>
  {
    private Bus _bus;
    private WindowService _windowService;

    public WindowDestroyedHandler(Bus bus, WindowService windowService)
    {
      _bus = bus;
      _windowService = windowService;
    }

    public void Handle(WindowDestroyedEvent @event)
    {
      var window = _windowService.GetWindows()
        .FirstOrDefault(window => window.Hwnd == @event.WindowHandle);

      if (window == null)
        return;

      // If window is in tree, detach the removed window from its parent.
      _bus.Invoke(new RemoveWindowCommand(window));
      _bus.Invoke(new RedrawContainersCommand());
    }
  }
}
