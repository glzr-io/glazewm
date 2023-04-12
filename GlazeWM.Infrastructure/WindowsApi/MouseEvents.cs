using System;
using System.Diagnostics;
using System.Reactive.Disposables;
using System.Reactive.Linq;
using System.Reactive.Subjects;
using System.Runtime.InteropServices;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Infrastructure.WindowsApi
{
  public static class MouseEvents
  {
    public static IObservable<MouseMoveEvent> MouseMoves
    {
      get
      {
        var mouseEvents = new Subject<MouseMoveEvent>();

        var isMouseDown = false;
        var hookProc = new HookProc((nCode, wParam, lParam) =>
        {
          var details = (LowLevelMouseInputEventDetails)Marshal.PtrToStructure(
            lParam,
            typeof(LowLevelMouseInputEventDetails)
          );

          // Check if mouse click is being held.
          switch ((WMessages)wParam)
          {
            case WMessages.WM_LBUTTONUP:
              isMouseDown = false;
              break;
            case WMessages.WM_LBUTTONDOWN:
              isMouseDown = true;
              break;
          }

          var filteredEvent = new MouseMoveEvent(details.pt, isMouseDown, details.TimeStamp);

          mouseEvents.OnNext(filteredEvent);

          return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);
        });

        var hookId = CreateHook(hookProc);

        return Observable.Create<MouseMoveEvent>(observer =>
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
