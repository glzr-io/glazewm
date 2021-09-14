using System.Linq;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;
using LarsWM.Infrastructure.WindowsApi.Events;

namespace LarsWM.Domain.Windows.EventHandlers
{
  class WindowFocusedHandler : IEventHandler<WindowFocusedEvent>
  {
    private Bus _bus;
    private WindowService _windowService;

    public WindowFocusedHandler(Bus bus, WindowService windowService)
    {
      _bus = bus;
      _windowService = windowService;
    }

    public void Handle(WindowFocusedEvent @event)
    {
      var window = _windowService.GetWindows()
        .FirstOrDefault(window => window.Hwnd == @event.WindowHandle);

      if (window == null)
        return;

      _bus.Invoke(new FocusWindowCommand(window));
    }
  }
}
