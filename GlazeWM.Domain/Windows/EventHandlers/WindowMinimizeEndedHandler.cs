using System.Linq;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi.Events;

namespace GlazeWM.Domain.Windows.EventHandlers
{
  class WindowMinimizeEndedHandler : IEventHandler<WindowMinimizeEndedEvent>
  {
    private Bus _bus;
    private WindowService _windowService;
    private ContainerService _containerService;

    public WindowMinimizeEndedHandler(Bus bus, WindowService windowService, ContainerService containerService)
    {
      _bus = bus;
      _windowService = windowService;
      _containerService = containerService;
    }

    public void Handle(WindowMinimizeEndedEvent @event)
    {
      var window = _windowService.GetWindows()
        .FirstOrDefault(window => window.Hwnd == @event.WindowHandle);

      if (window == null)
        return;

      // TODO: Create `MinimizedWindow` instance.

      _containerService.SplitContainersToRedraw.Add(window.Parent as SplitContainer);
      _bus.Invoke(new RedrawContainersCommand());
    }
  }
}
