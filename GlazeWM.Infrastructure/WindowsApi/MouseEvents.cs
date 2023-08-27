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

        var isRMouseDown = false;
        var isLMouseDown = false;
        var hookProc = new HookProc((nCode, wParam, lParam) =>
        {
          var details = (LowLevelMouseInputEventDetails)Marshal.PtrToStructure(
            lParam,
            typeof(LowLevelMouseInputEventDetails)
          );

          // Check if mouse click is being held.
          switch ((WMessages)wParam)
          {
            case WMessages.WM_RBUTTONUP:
              isRMouseDown = false;
              break;
            case WMessages.WM_RBUTTONDOWN:
              isRMouseDown = true;
              break;
            case WMessages.WM_LBUTTONUP:
              isLMouseDown = false;
              break;
            case WMessages.WM_LBUTTONDOWN:
              isLMouseDown = true;
              break;
          }

          var filteredEvent = new MouseMoveEvent(details.pt, isRMouseDown || isLMouseDown, details.TimeStamp);
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
