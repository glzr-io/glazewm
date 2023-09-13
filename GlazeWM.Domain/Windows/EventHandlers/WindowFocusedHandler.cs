using System.Linq;
using GlazeWM.Domain.Common.Utils;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Events;
using Microsoft.Extensions.Logging;

namespace GlazeWM.Domain.Windows.EventHandlers
{
  internal sealed class WindowFocusedHandler : IEventHandler<WindowFocusedEvent>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;
    private readonly ILogger<WindowFocusedHandler> _logger;
    private readonly WindowService _windowService;

    public WindowFocusedHandler(
      Bus bus,
      ContainerService containerService,
      ILogger<WindowFocusedHandler> logger,
      WindowService windowService)
    {
      _bus = bus;
      _containerService = containerService;
      _logger = logger;
      _windowService = windowService;
    }

    public void Handle(WindowFocusedEvent @event)
    {
      var windowHandle = @event.WindowHandle;
      var pendingFocusContainer = _containerService.PendingFocusContainer;

      // Override the container to set focus to (ie. when changing focus after a window is
      // closed or hidden).
      // TODO: This can be simplified a lot. Might also need to handle case where window
      // is null or not displayed.
      if (pendingFocusContainer is not null)
      {
        // If the container gaining focus is the pending focus container, then reset it.
        if (pendingFocusContainer is Window && @event.WindowHandle == (pendingFocusContainer as Window).Handle)
        {
          _bus.Invoke(new SetFocusedDescendantCommand(pendingFocusContainer));
          _bus.Emit(new FocusChangedEvent(pendingFocusContainer));
          _containerService.PendingFocusContainer = null;
          return;
        }

        var isDesktopWindow = windowHandle == _windowService.DesktopWindowHandle;

        if (pendingFocusContainer is Workspace && isDesktopWindow)
        {
          _bus.Invoke(new SetFocusedDescendantCommand(pendingFocusContainer));
          _bus.Emit(new FocusChangedEvent(pendingFocusContainer));
          _containerService.PendingFocusContainer = null;
          return;
        }

        _bus.InvokeAsync(new SyncNativeFocusCommand());
        return;
      }

      var window = _windowService.GetWindows()
        .FirstOrDefault(window => window.Handle == @event.WindowHandle);

      if (window is null || window?.IsDisplayed == false)
        return;

      _logger.LogWindowEvent("Native focus event", window);

      var focusedContainer = _containerService.FocusedContainer;

      // Focus is already set to the WM's focused container.
      if (window == focusedContainer)
        return;

      // Update the WM's focus state.
      _bus.Invoke(new SetFocusedDescendantCommand(window));
      _bus.Emit(new FocusChangedEvent(window));
    }
  }
}
