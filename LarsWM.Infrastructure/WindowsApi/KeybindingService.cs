using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Reactive.Subjects;
using System.Runtime.InteropServices;
using System.Threading;
using System.Windows.Forms;
using LarsWM.Infrastructure.Bussing;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Infrastructure.WindowsApi
{
  public class KeybindingService
  {
    private Subject<Keys> _modKeypresses = new Subject<Keys>();
    public static readonly uint WM_KEYDOWN = 0x100;
    public static readonly uint WM_SYSKEYDOWN = 0x104;

    private Bus _bus;

    public KeybindingService(Bus bus)
    {
      _bus = bus;
    }

    public void Start()
    {
      var thread = new Thread(() => CreateKeybindingHook());
      thread.Name = "LarsWMKeybindingService";
      thread.Start();
    }

    public void SetModKey(string modKey)
    {
      throw new NotImplementedException();
    }

    public void AddGlobalKeybinding(string keyCombination, Action callback)
    {
      throw new NotImplementedException();
    }

    private void CreateKeybindingHook()
    {
      SetWindowsHookEx(HookType.WH_KEYBOARD_LL, KeybindingHookProc, Process.GetCurrentProcess().MainModule.BaseAddress, 0);

      // `SetWindowsHookEx` requires a message loop within the thread that is executing the code.
      Application.Run();
    }

    private IntPtr KeybindingHookProc(int nCode, IntPtr wParam, IntPtr lParam)
    {
      // If nCode is less than zero, the hook procedure must return the value returned by CallNextHookEx.
      // CallNextHookEx passes hook notification to other applications.
      // TODO: Flatten this if-statement.
      if (nCode == 0 && ((uint)wParam == WM_KEYDOWN || (uint)wParam == WM_SYSKEYDOWN))
      {
        var modifiersPressed = new List<Keys>();

        if ((GetKeyState(Keys.LMenu) & 0x8000) == 0x8000)
          modifiersPressed.Add(Keys.LMenu);

        if (modifiersPressed.Count() == 0)
          return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);

        // Get struct with details about keyboard input event.
        var rawEventStruct = Marshal.PtrToStructure(lParam, typeof(LowLevelKeyboardInputEvent));
        var inputEvent = (LowLevelKeyboardInputEvent)rawEventStruct;

        _modKeypresses.OnNext(inputEvent.Key);

        // Avoid forwarding the key input to other applications.
        return new IntPtr(1);
      }

      return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);
    }
  }
}
