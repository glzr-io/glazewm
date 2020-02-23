using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;
using LarsWM.Infrastructure.WindowsApi;
using LarsWM.Infrastructure.WindowsApi.Enums;
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Text;

namespace LarsWM.Domain.Windows
{
    // TODO: Rename to WindowService once respository classes are created.
    public class WindowHooksHandler
    {
        private IBus _bus;
        private WindowService _windowService;
        private WindowEventService _windowEventService { get; }

        public WindowHooksHandler(IBus bus, WindowService windowService, WindowEventService windowEventService)
        {
            _bus = bus;
            _windowService = windowService;
            _windowEventService = windowEventService;
        }

        public void Configure()
        {
            _windowEventService.WindowHookSubject.Subscribe(observer =>
            {
                //var window = _windowService.GetWindowByHandle(observer.AffectedWindowHandle);

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
                        //_bus.Invoke(new FocusWindowCommand(window));
                        break;
                }
            });
        }
    }
}
