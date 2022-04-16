using System.Linq;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi.Events;

namespace GlazeWM.Domain.Windows.EventHandlers
{
  class WindowFocusedHandler : IEventHandler<WindowFocusedEvent>
  {
    private readonly Bus _bus;
    private readonly WindowService _windowService;
    private readonly ContainerService _containerService;

    public WindowFocusedHandler(Bus bus, WindowService windowService, ContainerService containerService)
    {
      _bus = bus;
      _windowService = windowService;
      _containerService = containerService;
    }

    public void Handle(WindowFocusedEvent @event)
    {
      var pendingFocusContainer = _containerService.PendingFocusContainer;

      // Override the container to set focus to (ie. when changing focus after a window is closed).
      if (pendingFocusContainer != null)
      {
        if (pendingFocusContainer is Window)
          Bus.Invoke(new FocusWindowCommand(pendingFocusContainer as Window));

        else if (pendingFocusContainer is Workspace)
          Bus.Invoke(new FocusWorkspaceCommand((pendingFocusContainer as Workspace).Name));

        _containerService.PendingFocusContainer = null;
        return;
      }

      var window = _windowService.GetWindows()
        .FirstOrDefault(window => window.Hwnd == @event.WindowHandle);

      if (window == null)
        return;

      Bus.Invoke(new SetFocusedDescendantCommand(window));
      _bus.RaiseEvent(new FocusChangedEvent(window));
    }
  }
}
