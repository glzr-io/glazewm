using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Events;
using GlazeWM.Infrastructure.WindowsApi.Enums;
using System;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Infrastructure.WindowsApi
{
  public class WindowEventService
  {
    private readonly Bus _bus;

    /// <summary>
    /// Store a reference to the hook delegate to prevent its garbage collection.
    /// </summary>
    private readonly WindowEventProc _hookProc;
    private const int CHILDID_SELF = 0;

    public WindowEventService(Bus bus)
    {
      _bus = bus;
      _hookProc = new WindowEventProc(WindowEventHookProc);
    }

    public void Start()
    {
      SetWinEventHook(EventConstant.EVENT_OBJECT_LOCATIONCHANGE, EventConstant.EVENT_OBJECT_LOCATIONCHANGE, IntPtr.Zero, _hookProc, 0, 0, 0);
      SetWinEventHook(EventConstant.EVENT_OBJECT_DESTROY, EventConstant.EVENT_OBJECT_HIDE, IntPtr.Zero, _hookProc, 0, 0, 0);
      SetWinEventHook(EventConstant.EVENT_SYSTEM_MINIMIZESTART, EventConstant.EVENT_SYSTEM_MINIMIZEEND, IntPtr.Zero, _hookProc, 0, 0, 0);
      SetWinEventHook(EventConstant.EVENT_SYSTEM_MOVESIZEEND, EventConstant.EVENT_SYSTEM_MOVESIZEEND, IntPtr.Zero, _hookProc, 0, 0, 0);
      SetWinEventHook(EventConstant.EVENT_SYSTEM_FOREGROUND, EventConstant.EVENT_SYSTEM_FOREGROUND, IntPtr.Zero, _hookProc, 0, 0, 0);
    }

    private void WindowEventHookProc(
      IntPtr hWinEventHook,
      EventConstant eventType,
      IntPtr hwnd,
      ObjectIdentifier idObject,
      int idChild,
      uint dwEventThread,
      uint dwmsEventTime)
    {
      // Whether the event is actually associated with a window object (instead of a UI control).
      var isWindowEvent = idChild == CHILDID_SELF && idObject == ObjectIdentifier.OBJID_WINDOW
        && hwnd != IntPtr.Zero;

      if (!isWindowEvent)
        return;

      Event eventToRaise = eventType switch
      {
        EventConstant.EVENT_OBJECT_LOCATIONCHANGE => new WindowLocationChangedEvent(hwnd),
        EventConstant.EVENT_SYSTEM_FOREGROUND => new WindowFocusedEvent(hwnd),
        EventConstant.EVENT_SYSTEM_MINIMIZESTART => new WindowMinimizedEvent(hwnd),
        EventConstant.EVENT_SYSTEM_MINIMIZEEND => new WindowMinimizeEndedEvent(hwnd),
        EventConstant.EVENT_SYSTEM_MOVESIZEEND => new WindowMovedOrResizedEvent(hwnd),
        EventConstant.EVENT_OBJECT_DESTROY => new WindowDestroyedEvent(hwnd),
        EventConstant.EVENT_OBJECT_SHOW => new WindowShownEvent(hwnd),
        EventConstant.EVENT_OBJECT_HIDE => new WindowHiddenEvent(hwnd),
        _ => null,
      };

      if (eventToRaise is not null)
        _bus.EmitAsync((dynamic)eventToRaise);
    }
  }
}
