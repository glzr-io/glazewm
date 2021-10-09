using System.Linq;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi.Events;

namespace GlazeWM.Domain.Windows.EventHandlers
{
  class WindowMovedOrResizedHandler : IEventHandler<WindowMovedOrResizedEvent>
  {
    private Bus _bus;
    private WindowService _windowService;

    public WindowMovedOrResizedHandler(Bus bus, WindowService windowService)
    {
      _bus = bus;
      _windowService = windowService;
    }

    public void Handle(WindowMovedOrResizedEvent @event)
    {
      var window = _windowService.GetWindows()
        .FirstOrDefault(window => window.Hwnd == @event.WindowHandle);

      if (window == null || !(window is FloatingWindow))
        return;

      // Update state with new location of the floating window.
      var updatedLocation = window.Location;
      window.X = updatedLocation.Left;
      window.Y = updatedLocation.Top;
      window.Height = updatedLocation.Bottom - updatedLocation.Top;
      window.Width = updatedLocation.Right - updatedLocation.Left;
    }
  }
}
