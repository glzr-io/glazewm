using System.Linq;
using LarsWM.Domain.Containers;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Domain.Workspaces;
using LarsWM.Domain.Workspaces.Commands;
using LarsWM.Infrastructure.Bussing;
using LarsWM.Infrastructure.WindowsApi.Events;

namespace LarsWM.Domain.Windows.EventHandlers
{
  class WindowFocusedHandler : IEventHandler<WindowFocusedEvent>
  {
    private Bus _bus;
    private WindowService _windowService;
    private ContainerService _containerService;

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
          _bus.Invoke(new FocusWindowCommand(pendingFocusContainer as Window));

        else if (pendingFocusContainer is Workspace)
          _bus.Invoke(new FocusWorkspaceCommand((pendingFocusContainer as Workspace).Name));

        _containerService.PendingFocusContainer = null;
        return;
      }

      var window = _windowService.GetWindows()
        .FirstOrDefault(window => window.Hwnd == @event.WindowHandle);

      if (window == null)
        return;

      _bus.Invoke(new FocusWindowCommand(window));
    }
  }
}
