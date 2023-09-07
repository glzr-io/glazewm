using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Monitors.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Events;

namespace GlazeWM.Domain.Windows.EventHandlers
{
  internal sealed class WindowShownHandler : IEventHandler<WindowShownEvent>
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
        .FirstOrDefault(window => window.Handle == windowHandle);

      // Manage the window if it's manageable.
      if (window is null && WindowService.IsHandleManageable(windowHandle))
      {
        _bus.Invoke(new ManageWindowCommand(windowHandle));
        _bus.Invoke(new RedrawContainersCommand());
        _bus.Invoke(new SyncNativeFocusCommand());
        return;
      }

      // Update display state if window is already managed.
      if (window?.DisplayState == DisplayState.Showing)
        window.DisplayState = DisplayState.Shown;
    }
  }
}
