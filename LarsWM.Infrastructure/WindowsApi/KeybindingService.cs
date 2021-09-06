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
    public List<Keys> KeyCombination { get; }
    public Action KeybindingProc { get; }

    public Keybinding(List<Keys> keyCombination, Action keybindingProc)
    {
      KeyCombination = keyCombination;
      KeybindingProc = keybindingProc;
    }
  }

  public class KeybindingService
  {
    private static readonly uint WM_KEYDOWN = 0x100;
    private static readonly uint WM_SYSKEYDOWN = 0x104;
    private static readonly int WM_KEYUP = 0x0101;
    private static readonly int WM_SYSKEYUP = 0x0105;
    private List<Keybinding> _registeredKeybindings = new List<Keybinding>();
    private Keys _modKey = Keys.LMenu;
    private List<Keys> _triggerKeys = new List<Keys>();

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
      bindingString = bindingString.Replace("$mod", Enum.GetName(typeof(Keys), _modKey));

      var bindingParts = bindingString
        .Split('+')
        .Select(key => Enum.Parse(typeof(Keys), key))
        .Cast<Keys>()
        .ToList();

      var triggerKey = bindingParts[0];
      bindingParts.RemoveAt(0);

      _triggerKeys.Add(triggerKey);

      var keybinding = new Keybinding(bindingParts, callback);
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
      var passThrough = nCode != 0 || !((uint)wParam == WM_KEYDOWN || (uint)wParam == WM_SYSKEYDOWN);

      // If nCode is less than zero, the hook procedure must return the value returned by CallNextHookEx.
      // CallNextHookEx passes the hook notification to other applications.
      if (passThrough)
        return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);

      // TODO: Check whether any of the trigger keys were pressed.
      var modifiersPressed = new List<Keys>();

      if ((GetKeyState(Keys.LMenu) & 0x8000) == 0x8000)
        modifiersPressed.Add(Keys.LMenu);

      // if ((GetKeyState(Keys.LShiftKey) & 0x8000) == 0x8000)
      //   modifiersPressed.Add(Keys.LShiftKey);

      // if ((GetKeyState(Keys.LControlKey) & 0x8000) == 0x8000)
      //   modifiersPressed.Add(Keys.LControlKey);

      if (modifiersPressed.Count() == 0)
        return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);

      var state = GetCurrentKeyboardState();

      // var isModKeyPressed = IsDown(state, _modKey);

      // if (!isModKeyPressed)
      //   return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);

      var keysPressed = new List<Keys>();
      foreach (var key in Enum.GetValues(typeof(Keys)))
      {
        if (IsDown(state, (Keys)key))
        {
          keysPressed.Add((Keys)key);
          Debug.WriteLine("Key is pressed: " + key);
        }
      }

      if (IsDown(state, Keys.LMenu & Keys.J))
        Debug.WriteLine("Key is pressed: Keys.LMenu | Keys.J");

      foreach (var keybinding in _registeredKeybindings)
      {
        var isMatch = keybinding.KeyCombination.All(key => IsDown(state, key));

        if (isMatch)
          keybinding.KeybindingProc();
      }

      // Avoid forwarding the key input to other applications.
      return new IntPtr(1);
      // return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);
    }

    private byte[] GetCurrentKeyboardState()
    {
      var keyboardState = new byte[256];
      GetKeyboardState(keyboardState);
      return keyboardState;
    }

    /// <summary>
    /// Whether the specified key is down in a keyboard state snapshot.
    /// </summary>
    private bool IsDown(byte[] state, Keys key)
    {
      if ((int)key < 256) return IsDownRaw(state, key);
      if (key == Keys.Alt) return IsDownRaw(state, Keys.LMenu) || IsDownRaw(state, Keys.RMenu);
      if (key == Keys.Shift) return IsDownRaw(state, Keys.LShiftKey) || IsDownRaw(state, Keys.RShiftKey);
      if (key == Keys.Control) return IsDownRaw(state, Keys.LControlKey) || IsDownRaw(state, Keys.RControlKey);
      return false;
    }

    private bool IsDownRaw(byte[] state, Keys key)
    {
      var virtualKeyCode = (int)key;

      // if (virtualKeyCode < 0 || virtualKeyCode > 255)
      //   throw new ArgumentOutOfRangeException("key", key, "The value must be between 0 and 255.");

      var keyState = state.ElementAtOrDefault(virtualKeyCode);

      // if (keyState == null)
      //   return false;

      // Get the high-order bit. If the high-order bit is 1, the key is down.
      return keyState >> 7 == 1;
    }
  }
}
