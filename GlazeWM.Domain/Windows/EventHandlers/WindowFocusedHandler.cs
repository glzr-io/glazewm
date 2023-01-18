using System.Linq;
using GlazeWM.Domain.Common.Utils;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Events;
using Microsoft.Extensions.Logging;

namespace GlazeWM.Domain.Windows.EventHandlers
{
  internal sealed class WindowFocusedHandler : IEventHandler<WindowFocusedEvent>
  {
    private readonly Bus _bus;
    private readonly WindowService _windowService;
    private readonly ILogger<WindowFocusedHandler> _logger;

    public WindowFocusedHandler(
      Bus bus,
      WindowService windowService,
      ILogger<WindowFocusedHandler> logger)
    {
      _bus = bus;
      _windowService = windowService;
      _logger = logger;
    }

    public void Handle(WindowFocusedEvent @event)
    {
      var window = _windowService.GetWindows()
        .FirstOrDefault(window => window.Handle == @event.WindowHandle);

      if (window is null || window?.IsDisplayed == false)
        return;

      _logger.LogWindowEvent("Window focused", window);

      _bus.Invoke(new SetFocusedDescendantCommand(window));
      _bus.Emit(new FocusChangedEvent(window));
    }
  }
}
