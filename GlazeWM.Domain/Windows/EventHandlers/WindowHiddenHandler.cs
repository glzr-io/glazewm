using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi.Events;

namespace GlazeWM.Domain.Windows.EventHandlers
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
      if (window == null || window.IsHidden == true || _windowService.IsHandleVisible(window.Hwnd))
        return;

      // Detach the hidden window from its parent.
      _bus.Invoke(new RemoveWindowCommand(window));
      _bus.Invoke(new RedrawContainersCommand());
    }
  }
}
