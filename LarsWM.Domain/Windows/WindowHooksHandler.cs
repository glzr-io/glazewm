using System;
using System.Diagnostics;
using System.Linq;
using LarsWM.Domain.Containers;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;
using LarsWM.Infrastructure.WindowsApi;
using LarsWM.Infrastructure.WindowsApi.Enums;

namespace LarsWM.Domain.Windows
{
  // TODO: Move Configure method to WindowService.
  public class WindowHooksHandler
  {
    private IBus _bus;
    private WindowService _windowService;
    private ContainerService _containerService;

    private WindowEventService _windowEventService { get; }

    public WindowHooksHandler(
        IBus bus,
        WindowService windowService,
        WindowEventService windowEventService,
        ContainerService containerService)
    {
      _bus = bus;
      _windowService = windowService;
      _windowEventService = windowEventService;
      _containerService = containerService;
    }

    public void Configure()
    {
      _windowEventService.WindowHookSubject.Subscribe(observer =>
      {
        // TODO: For performance, instead get window instance by using
        // MonitorService.GetMonitorFromUnaddedWindow and searching its displayed
        // workspace.
        var window = _windowService.GetWindows()
            .FirstOrDefault(w => w.Hwnd == observer.AffectedWindowHandle);

        switch (observer.EventType)
        {
          case EventConstant.EVENT_OBJECT_SHOW:
            Debug.WriteLine("show");
            // Don't invoke AddWindowCommand if window is already in tree.
            if (window != null)
              return;

            _bus.Invoke(new AddWindowCommand(observer.AffectedWindowHandle));
            break;
          case EventConstant.EVENT_OBJECT_DESTROY:
            Debug.WriteLine("destroy");
            // If window is in tree, detach removed window from its parent.
            if (window != null)
            {
              _bus.Invoke(new DetachContainerCommand(window.Parent as SplitContainer, window));
              _bus.Invoke(new RedrawContainersCommand());
            }

            break;
          case EventConstant.EVENT_OBJECT_CLOAKED:
            Debug.WriteLine("cloaked");
            break;
          case EventConstant.EVENT_OBJECT_UNCLOAKED:
            Debug.WriteLine("decloaked");
            break;
          case EventConstant.EVENT_SYSTEM_MINIMIZESTART:
            Debug.WriteLine("minimize start");
            break;
          case EventConstant.EVENT_SYSTEM_MINIMIZEEND:
            Debug.WriteLine("minimize end");
            break;
          case EventConstant.EVENT_SYSTEM_FOREGROUND:
            Debug.WriteLine("foreground");
            if (window == null)
              return;

            _bus.Invoke(new FocusWindowCommand(window));
            break;
        }
      });
    }
  }
}
