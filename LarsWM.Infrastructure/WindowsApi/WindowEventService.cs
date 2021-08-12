using LarsWM.Infrastructure.WindowsApi.Enums;
using System;
using System.Reactive.Subjects;
using System.Threading;
using System.Windows.Forms;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Infrastructure.WindowsApi
{
  public class WindowEventService
  {
    // TODO: Change from Subject to Observable.
    // TODO: Create the Observable in constructor instead of `Init` method.
    public Subject<WindowHookEvent> WindowHookSubject = new Subject<WindowHookEvent>();

    public static int CHILDID_SELF = 0;

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

    public void Init()
    {
      var callback = new WindowEventProc((IntPtr hWinEventHook, EventConstant eventType, IntPtr hwnd, ObjectIdentifier idObject, int idChild, uint dwEventThread, uint dwmsEventTime) =>
      {
        var isWindowEvent = idChild == CHILDID_SELF && idObject == ObjectIdentifier.OBJID_WINDOW && hwnd != IntPtr.Zero;

        if (!isWindowEvent)
          return;

        var windowHookEvent = new WindowHookEvent(eventType, hwnd);
        WindowHookSubject.OnNext(windowHookEvent);
      });

      // SetWinEventHook requires a message loop within the thread that is executing the code.
      var thread = new Thread(() =>
      {
        SetWinEventHook(EventConstant.EVENT_OBJECT_DESTROY, EventConstant.EVENT_OBJECT_SHOW, IntPtr.Zero, callback, 0, 0, 0);
        SetWinEventHook(EventConstant.EVENT_OBJECT_CLOAKED, EventConstant.EVENT_OBJECT_UNCLOAKED, IntPtr.Zero, callback, 0, 0, 0);
        SetWinEventHook(EventConstant.EVENT_SYSTEM_MINIMIZESTART, EventConstant.EVENT_SYSTEM_MINIMIZEEND, IntPtr.Zero, callback, 0, 0, 0);
        SetWinEventHook(EventConstant.EVENT_SYSTEM_FOREGROUND, EventConstant.EVENT_SYSTEM_FOREGROUND, IntPtr.Zero, callback, 0, 0, 0);
        Application.Run();
      });
      thread.Name = "LarsWMWindowHooks";
      thread.Start();

    }
  }
}
