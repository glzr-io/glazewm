using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Reactive.Disposables;
using System.Reactive.Linq;
using System.Reactive.Subjects;
using System.Reactive.Threading.Tasks;
using System.Runtime.InteropServices;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Infrastructure.WindowsApi
{
  public static class MouseEvents
  {
    /// <summary>
    /// Hwnd of currently focused window and it's known children
    /// </summary>
    private static List<IntPtr> FocusedWindows = new();

    public static IObservable<LowLevelMouseInputEvent> MouseMoves
    {
      get
      {
        var mouseEvents = new Subject<LowLevelMouseInputEvent>();

        var hookProc = new HookProc((nCode, wParam, lParam) =>
        {
          var inputEvent = (LowLevelMouseInputEvent)Marshal.PtrToStructure(
            lParam,
            typeof(LowLevelMouseInputEvent)
          );

          mouseEvents.OnNext(inputEvent);

          // Returns window underneath cursor.  This could be a child window or parent.
          var window = WindowFromPoint(inputEvent.pt);

          // If the mouse is hovering over the currently focused main window or one of it's children, do nothing.
          if (FocusedWindows.Contains(window))
            return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);

          // If the FocusedWindows list didn't contain the window, this must be a new window being focused.
          FocusedWindows.Clear();
          FocusedWindows.Add(window);

          // Check if the window is the main window or a child window.
          var parentWindow = GetParent(window);

          // Walk the window up each parent window until you have the main window.
          while (parentWindow != IntPtr.Zero)
          {
            window = parentWindow;
            FocusedWindows.Add(window);
            parentWindow = GetParent(window);
          }

          // Focus the main window
          SetForegroundWindow(window);
          SetFocus(window);

          return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);
        });

        var hookId = CreateHook(hookProc);

        return Observable.Create<LowLevelMouseInputEvent>(observer =>
        {
          var subscription = mouseEvents.Subscribe(
            mouseEvent => observer.OnNext(mouseEvent)
          );

          return Disposable.Create(() =>
          {
            // Unregister mouse hook on observable completion.
            subscription.Dispose();
            UnhookWindowsHookEx(hookId);
            GC.KeepAlive(hookProc);
          });
        });
      }
    }

    /// <summary>
    /// Create a low-level mouse hook.
    /// </summary>
    private static IntPtr CreateHook(HookProc proc)
    {
      return SetWindowsHookEx(
        HookType.MouseLowLevel,
        proc,
        Process.GetCurrentProcess().MainModule.BaseAddress,
        0
      );
    }
  }
}