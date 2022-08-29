using System.Linq;
using GlazeWM.Domain.Common.Utils;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi.Events;
using Microsoft.Extensions.Logging;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows.EventHandlers
{
  internal class WindowFocusedHandler : IEventHandler<WindowFocusedEvent>
  {
    private readonly Bus _bus;
    private readonly WindowService _windowService;
    private readonly ContainerService _containerService;
    private readonly ILogger<WindowFocusedHandler> _logger;

    public WindowFocusedHandler(
      Bus bus,
      WindowService windowService,
      ContainerService containerService,
      ILogger<WindowFocusedHandler> logger
    )
    {
      _bus = bus;
      _windowService = windowService;
      _containerService = containerService;
      _logger = logger;
    }

    public void Handle(WindowFocusedEvent @event)
    {
      var pendingFocusContainer = _containerService.PendingFocusContainer;

      // Override the container to set focus to (ie. when changing focus after a window is closed).
      if (pendingFocusContainer != null)
      {
        if (pendingFocusContainer is Window)
          _bus.Invoke(new FocusWindowCommand(pendingFocusContainer as Window));
        else if (pendingFocusContainer is Workspace)
        {
          _bus.Invoke(new SetFocusedDescendantCommand(pendingFocusContainer));
          KeybdEvent(0, 0, 0, 0);
          SetForegroundWindow(GetDesktopWindow());
        }

        _containerService.PendingFocusContainer = null;
        return;
      }

      var window = _windowService.GetWindows()
        .FirstOrDefault(window => window.Hwnd == @event.WindowHandle);

      if (window == null)
        return;

      _logger.LogWindowEvent("Window focused", window);

      _bus.Invoke(new SetFocusedDescendantCommand(window));
      _bus.RaiseEvent(new FocusChangedEvent(window));
    }
  }
}
