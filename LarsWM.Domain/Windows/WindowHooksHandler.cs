using LarsWM.Domain.Containers;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;
using LarsWM.Infrastructure.WindowsApi;
using LarsWM.Infrastructure.WindowsApi.Enums;
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Text;

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
                var window = _containerService.ContainerTree.TraverseDownEnumeration()
                    .OfType<Window>()
                    .FirstOrDefault(w => w.Hwnd == observer.AffectedWindowHandle);

                switch (observer.EventType)
                {
                    case EventConstant.EVENT_OBJECT_SHOW:
                        Debug.WriteLine("show");
                        break;
                    case EventConstant.EVENT_OBJECT_DESTROY:
                        Debug.WriteLine("destroy");
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
                        _bus.Invoke(new FocusWindowCommand(window));
                        break;
                }
            });
        }
    }
}
