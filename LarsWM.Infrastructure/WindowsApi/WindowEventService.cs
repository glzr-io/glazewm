using LarsWM.Infrastructure.WindowsApi.Enums;
using System;
using System.Reactive.Subjects;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Infrastructure.WindowsApi
{
    class WindowEventService
    {
        public Subject<WindowHookEvent> WindowHookSubject = new Subject<WindowHookEvent>();

        public struct WindowHookEvent
        {
            public EventConstant EventType { get; }
            public IntPtr AffectedWindowHandle { get; }

            public WindowHookEvent(EventConstant eventType, IntPtr hwnd)
            {
                EventType = eventType;
                AffectedWindowHandle = hwnd;
            }
        }

        public void Initialise()
        {
            var callback = new WindowEventProc((IntPtr hWinEventHook, EventConstant eventType, IntPtr hwnd, ObjectIdentifier idObject, int idChild, uint dwEventThread, uint dwmsEventTime) =>
            {
                var CHILDID_SELF = 0;
                var validEvent = idChild == CHILDID_SELF && idObject == ObjectIdentifier.OBJID_WINDOW && hwnd != IntPtr.Zero;

                if (!validEvent)
                    return;

                var windowHookEvent = new WindowHookEvent(eventType, hwnd);
                WindowHookSubject.OnNext(windowHookEvent);
            });

            SetWinEventHook(EventConstant.EVENT_OBJECT_DESTROY, EventConstant.EVENT_OBJECT_SHOW, IntPtr.Zero, callback, 0, 0, 0);
            SetWinEventHook(EventConstant.EVENT_OBJECT_CLOAKED, EventConstant.EVENT_OBJECT_UNCLOAKED, IntPtr.Zero, callback, 0, 0, 0);
            SetWinEventHook(EventConstant.EVENT_SYSTEM_MINIMIZESTART, EventConstant.EVENT_SYSTEM_MINIMIZEEND, IntPtr.Zero, callback, 0, 0, 0);
            SetWinEventHook(EventConstant.EVENT_SYSTEM_FOREGROUND, EventConstant.EVENT_SYSTEM_FOREGROUND, IntPtr.Zero, callback, 0, 0, 0);
        }
    }
}
