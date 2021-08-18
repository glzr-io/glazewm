using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Reactive.Subjects;
using System.Runtime.InteropServices;
using System.Threading;
using System.Windows.Forms;
using LarsWM.Domain.Common.Enums;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Domain.Workspaces.Commands;
using LarsWM.Infrastructure.Bussing;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Domain.Common.Services
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

    public void Init()
    {
      // SetWindowsHookEx requires a message loop within the thread that is executing the code.
      var thread = new Thread(() =>
      {
        SetWindowsHookEx(HookType.WH_KEYBOARD_LL, KbHookProc, Process.GetCurrentProcess().MainModule.BaseAddress, 0);
        //SetWindowsHookEx(HookType.WH_MOUSE_LL, _mouseHook, Process.GetCurrentProcess().MainModule.BaseAddress, 0);
        Application.Run();
      });
      thread.Name = "LarsWMKeybindingService";
      thread.Start();

      HandleKeybindings();
    }


    private IntPtr KbHookProc(int nCode, IntPtr wParam, IntPtr lParam)
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

        // Get structure with details about keyboard input event.
        var hookStruct = (KbLLHookStruct)Marshal.PtrToStructure(lParam, typeof(KbLLHookStruct));

        _modKeypresses.OnNext(hookStruct.vkCode);
        return new IntPtr(1);
      }

      return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);
    }

    private void HandleKeybindings()
    {
      _modKeypresses.Subscribe(pressedKey =>
      {
        if (pressedKey == Keys.J)
          Debug.WriteLine("Alt+J keybinding successfully triggered.");
        if (pressedKey == Keys.D1)
          _bus.Invoke(new FocusWorkspaceCommand("1"));
        if (pressedKey == Keys.D2)
          _bus.Invoke(new FocusWorkspaceCommand("2"));
        if (pressedKey == Keys.D3)
          _bus.Invoke(new FocusWorkspaceCommand("3"));
        if (pressedKey == Keys.D4)
          _bus.Invoke(new FocusWorkspaceCommand("4"));
        if (pressedKey == Keys.D5)
          _bus.Invoke(new FocusWorkspaceCommand("5"));
        if (pressedKey == Keys.D6)
          _bus.Invoke(new FocusWorkspaceCommand("6"));
        if (pressedKey == Keys.D7)
          _bus.Invoke(new FocusWorkspaceCommand("7"));
        if (pressedKey == Keys.D8)
          _bus.Invoke(new FocusWorkspaceCommand("8"));
        if (pressedKey == Keys.D9)
          _bus.Invoke(new FocusWorkspaceCommand("9"));
        if (pressedKey == Keys.U)
          _bus.Invoke(new ResizeFocusedWindowCommand(ResizeDirection.SHRINK_WIDTH));
        if (pressedKey == Keys.P)
          _bus.Invoke(new ResizeFocusedWindowCommand(ResizeDirection.GROW_WIDTH));
        if (pressedKey == Keys.V)
          _bus.Invoke(new ChangeContainerLayoutCommand(Layout.Vertical));
        if (pressedKey == Keys.B)
          _bus.Invoke(new ChangeContainerLayoutCommand(Layout.Horizontal));
      }
      );
    }
  }
}
