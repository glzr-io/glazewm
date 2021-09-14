using System.Linq;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;
using LarsWM.Infrastructure.WindowsApi.Events;

namespace LarsWM.Domain.Windows.EventHandlers
{
  class WindowHiddenHandler : IEventHandler<WindowHiddenEvent>
  {
    private Bus _bus;
    private WindowService _windowService;

    public WindowHiddenHandler(Bus bus, WindowService windowService)
    {
      _bus = bus;
      _windowService = windowService;
    }

    public void Handle(WindowHiddenEvent @event)
    {
      var window = _windowService.GetWindows()
        .FirstOrDefault(window => window.Hwnd == @event.WindowHandle);

      // Ignore cases where the window isn't managed or is actually supposed to be hidden.
      if (window == null || window.IsHidden == true)
        return;

      // Detach the hidden window from its parent.
      _bus.Invoke(new RemoveWindowCommand(window));
      _bus.Invoke(new RedrawContainersCommand());
    }
  }
}
