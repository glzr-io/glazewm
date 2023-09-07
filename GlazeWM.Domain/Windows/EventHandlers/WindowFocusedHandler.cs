using System.Linq;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Common.Utils;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Domain.Workspaces.Commands;
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

      var window = _windowService.GetWindows()
        .FirstOrDefault(window => window.Handle == @event.WindowHandle);

      if (window is null)
        return;

      _logger.LogWindowEvent("Native focus event", window);

      var focusedContainer = _containerService.FocusedContainer;

      // Focus is already set to the WM's focused container.
      if (window == focusedContainer)
        return;

      var unmanagedStopwatch = _windowService.UnmanagedOrMinimizedStopwatch;

      if (unmanagedStopwatch?.ElapsedMilliseconds < 100)
      {
        _logger.LogDebug("Overriding native focus.");
        _bus.Invoke(new SyncNativeFocusCommand());
        return;
      }

      // TODO: Need to return early for other display states.
      // TODO: Should this be moved to `WindowShownHandler`. Is show event
      // emitted first, or foreground?
      if (window.DisplayState is DisplayState.Hidden)
      {
        _logger.LogWindowEvent("Focusing off-screen window", window);
        var workspace = WorkspaceService.GetWorkspaceFromChildContainer(window);
        _bus.Invoke(new FocusWorkspaceCommand(workspace.Name));
      }

      // Update the WM's focus state.
      _bus.Invoke(new SetFocusedDescendantCommand(window));
      _bus.Emit(new FocusChangedEvent(window));
    }
  }
}
