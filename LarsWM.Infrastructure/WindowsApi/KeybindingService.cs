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
      var bindingParts = bindingString
        .Split('+')
        .Select(key => FormatKeybinding(key))
        .Select(key => Enum.Parse(typeof(Keys), key))
        .Cast<Keys>()
        .ToList();

      var keybinding = new Keybinding(bindingParts, callback);
      _registeredKeybindings.Add(keybinding);
    }

    private string FormatKeybinding(string key)
    {
      if (key == "$mod")
        return Enum.GetName(typeof(Keys), _modKey);

      var isNumeric = int.TryParse(key, out int _);

      if (isNumeric)
        return $"D{key}";

      return key;
    }

    private void CreateKeybindingHook()
    {
      SetWindowsHookEx(HookType.WH_KEYBOARD_LL, KeybindingHookProc, Process.GetCurrentProcess().MainModule.BaseAddress, 0);

      // `SetWindowsHookEx` requires a message loop within the thread that is executing the code.
      Application.Run();
    }

    private IntPtr KeybindingHookProc(int nCode, IntPtr wParam, IntPtr lParam)
    {
      var shouldPassThrough = nCode != 0 || !((uint)wParam == WM_KEYDOWN || (uint)wParam == WM_SYSKEYDOWN);

      // If nCode is less than zero, the hook procedure must pass the hook notification to other
      // applications via `CallNextHookEx`.
      if (shouldPassThrough)
        return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);

      // Get struct with details about keyboard input event.
      var inputEvent =
        (LowLevelKeyboardInputEvent)Marshal.PtrToStructure(lParam, typeof(LowLevelKeyboardInputEvent));

      var pressedKey = inputEvent.Key;

      Keybinding matchedKeybinding = null;

      foreach (var keybinding in _registeredKeybindings)
      {
        var isMatch = keybinding.KeyCombination.All(key => key == pressedKey || IsKeyDown(key));

        if (!isMatch)
          continue;

        // Replace the keybinding to call if the key combination is longer.
        if (matchedKeybinding == null || keybinding.KeyCombination.Count() > matchedKeybinding.KeyCombination.Count())
          matchedKeybinding = keybinding;
      }

      // Invoke the matched keybinding.
      if (matchedKeybinding != null)
      {
        matchedKeybinding.KeybindingProc();

        // Avoid forwarding the key input to other applications.
        return new IntPtr(1);
      }

      return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);
    }

    /// <summary>
    /// Whether the specified key is down.
    /// </summary>
    private bool IsKeyDown(Keys key)
    {
      if (key == Keys.Alt)
        return IsKeyDownRaw(Keys.LMenu) || IsKeyDownRaw(Keys.RMenu);

      if (key == Keys.Shift)
        return IsKeyDownRaw(Keys.LShiftKey) || IsKeyDownRaw(Keys.RShiftKey);

      if (key == Keys.Control)
        return IsKeyDownRaw(Keys.LControlKey) || IsKeyDownRaw(Keys.RControlKey);

      return IsKeyDownRaw(key);
    }

    private bool IsKeyDownRaw(Keys key)
    {
      return (GetKeyState(key) & 0x8000) == 0x8000;
    }
  }
}
