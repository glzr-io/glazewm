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
      var thread = new Thread(() => CreateWindowEventHook());
      thread.Name = "LarsWMWindowHooks";
      thread.Start();
    }

    private void CreateWindowEventHook()
    {
      SetWinEventHook(EventConstant.EVENT_OBJECT_DESTROY, EventConstant.EVENT_OBJECT_HIDE, IntPtr.Zero, WindowEventHookProc, 0, 0, 0);
      SetWinEventHook(EventConstant.EVENT_OBJECT_CLOAKED, EventConstant.EVENT_OBJECT_UNCLOAKED, IntPtr.Zero, WindowEventHookProc, 0, 0, 0);
      SetWinEventHook(EventConstant.EVENT_SYSTEM_MINIMIZESTART, EventConstant.EVENT_SYSTEM_MINIMIZEEND, IntPtr.Zero, WindowEventHookProc, 0, 0, 0);
      SetWinEventHook(EventConstant.EVENT_SYSTEM_FOREGROUND, EventConstant.EVENT_SYSTEM_FOREGROUND, IntPtr.Zero, WindowEventHookProc, 0, 0, 0);

      // `SetWindowsHookEx` requires a message loop within the thread that is executing the code.
      Application.Run();
    }

    private void WindowEventHookProc(IntPtr hWinEventHook, EventConstant eventType, IntPtr hwnd, ObjectIdentifier idObject, int idChild, uint dwEventThread, uint dwmsEventTime)
    {
      var isWindowEvent = idChild == CHILDID_SELF && idObject == ObjectIdentifier.OBJID_WINDOW && hwnd != IntPtr.Zero;

      if (!isWindowEvent)
        return;

      var windowHookEvent = new WindowHookEvent(eventType, hwnd);
      WindowHookSubject.OnNext(windowHookEvent);
    }
  }
}
