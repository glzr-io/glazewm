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
        private WindowEventService _windowEventService { get; }

        public WindowHooksHandler(WindowEventService windowEventService)
        {
            _windowEventService = windowEventService;
        }

        public void Configure()
        {
            _windowEventService.WindowHookSubject.Subscribe(observer =>
            {
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
                        break;
                }
            });
        }
    }
}
