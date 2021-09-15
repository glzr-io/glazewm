using System.Linq;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;
using LarsWM.Infrastructure.WindowsApi.Events;

namespace LarsWM.Domain.Windows.EventHandlers
{
  class WindowShownHandler : IEventHandler<WindowShownEvent>
  {
    private Bus _bus;
    private WindowService _windowService;

    public WindowShownHandler(Bus bus, WindowService windowService)
    {
      _bus = bus;
      _windowService = windowService;
    }

    public void Handle(WindowShownEvent @event)
    {
      var window = _windowService.GetWindows()
        .FirstOrDefault(window => window.Hwnd == @event.WindowHandle);

      // Ignore cases where window is already managed.
      if (window != null)
        return;

      _bus.Invoke(new AddWindowCommand(@event.WindowHandle));
    }
  }
}
