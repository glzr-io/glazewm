using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Runtime.InteropServices;
using System.Threading;
using System.Windows.Forms;
using LarsWM.Infrastructure.Bussing;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Infrastructure.WindowsApi
{
  internal class Keybinding
  {
    public Stack<Keys> KeyCombination { get; }
    public Action KeybindingProc { get; }

    public Keybinding(Stack<Keys> keyCombination, Action keybindingProc)
    {
      KeyCombination = keyCombination;
      KeybindingProc = keybindingProc;
    }
  }

  public class KeybindingService
  {
    public static readonly uint WM_KEYDOWN = 0x100;
    public static readonly uint WM_SYSKEYDOWN = 0x104;
    private List<Keybinding> _registeredKeybindings = new List<Keybinding>();
    private Keys _modKey = Keys.LMenu;

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
      _modKey = (Keys)Enum.Parse(typeof(Keys), modKey);
    }

    public void AddGlobalKeybinding(string bindingString, Action callback)
    {
      if (!bindingString.Contains("$mod"))
        throw new Exception($"Key combination '{bindingString}' must include $mod.");

      bindingString = bindingString.Replace("$mod", Enum.GetName(typeof(Keys), _modKey));

      var bindingParts = bindingString
        .Split('+')
        .Select(key => Enum.Parse(typeof(Keys), key))
        .Cast<Keys>();

      var stack = new Stack<Keys>(bindingParts);
      var triggerKey = stack.Pop();

      if (triggerKey != _modKey)
        throw new Exception($"Key combination '{bindingString}' must use $mod as trigger key.");

      var keybinding = new Keybinding(stack, callback);
      _registeredKeybindings.Add(keybinding);
    }

    private void CreateKeybindingHook()
    {
      SetWindowsHookEx(HookType.WH_KEYBOARD_LL, KeybindingHookProc, Process.GetCurrentProcess().MainModule.BaseAddress, 0);

      // `SetWindowsHookEx` requires a message loop within the thread that is executing the code.
      Application.Run();
    }

    private IntPtr KeybindingHookProc(int nCode, IntPtr wParam, IntPtr lParam)
    {
      var passThrough = nCode != 0;

      // If nCode is less than zero, the hook procedure must return the value returned by CallNextHookEx.
      // CallNextHookEx passes the hook notification to other applications.
      if (passThrough)
        return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);

      var modifiersPressed = new List<Keys>();

      if ((GetKeyState(Keys.LMenu) & 0x8000) == 0x8000)
        modifiersPressed.Add(Keys.LMenu);

      if (modifiersPressed.Count() == 0)
        return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);

      // Get struct with details about keyboard input event.
      var rawEventStruct = Marshal.PtrToStructure(lParam, typeof(LowLevelKeyboardInputEvent));
      var inputEvent = (LowLevelKeyboardInputEvent)rawEventStruct;

      // Avoid forwarding the key input to other applications.
      return new IntPtr(1);
    }
  }
}
