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
