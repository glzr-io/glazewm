using System;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Events;
using GlazeWM.Infrastructure.WindowsApi.Enums;
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
      SetWinEventHook(EventConstant.LocationChange, EventConstant.LocationChange, IntPtr.Zero, _hookProc, 0, 0, 0);
      SetWinEventHook(EventConstant.Destroy, EventConstant.Hide, IntPtr.Zero, _hookProc, 0, 0, 0);
      SetWinEventHook(EventConstant.MinimizeStart, EventConstant.MinimizeEnd, IntPtr.Zero, _hookProc, 0, 0, 0);
      SetWinEventHook(EventConstant.MoveSizeEnd, EventConstant.MoveSizeEnd, IntPtr.Zero, _hookProc, 0, 0, 0);
      SetWinEventHook(EventConstant.Foreground, EventConstant.Foreground, IntPtr.Zero, _hookProc, 0, 0, 0);
      SetWinEventHook(EventConstant.LocationChange, EventConstant.NameChange, IntPtr.Zero, _hookProc, 0, 0, 0);
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
      var isWindowEvent = idChild == CHILDID_SELF && idObject == ObjectIdentifier.Window
        && hwnd != IntPtr.Zero;

      if (!isWindowEvent)
        return;

      Event eventToRaise = eventType switch
      {
        EventConstant.LocationChange => new WindowLocationChangedEvent(hwnd),
        EventConstant.Foreground => new WindowFocusedEvent(hwnd),
        EventConstant.MinimizeStart => new WindowMinimizedEvent(hwnd),
        EventConstant.MinimizeEnd => new WindowMinimizeEndedEvent(hwnd),
        EventConstant.MoveSizeEnd => new WindowMovedOrResizedEvent(hwnd),
        EventConstant.Destroy => new WindowDestroyedEvent(hwnd),
        EventConstant.Show => new WindowShownEvent(hwnd),
        EventConstant.NameChange => new WindowTitleChangedEvent(hwnd),
        EventConstant.Hide => new WindowHiddenEvent(hwnd),
        _ => null,
      };

      if (eventToRaise is not null)
        _bus.EmitAsync((dynamic)eventToRaise);
    }
  }
}
