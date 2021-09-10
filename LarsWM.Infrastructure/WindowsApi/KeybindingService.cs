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
    private Bus _bus;
    private static readonly uint WM_KEYDOWN = 0x100;
    private static readonly uint WM_SYSKEYDOWN = 0x104;
    private Keys _modKey = Keys.LMenu;

    /// <summary>
    /// Registered keybindings grouped by trigger key (ie. the final key in a key combination).
    /// </summary>
    private Dictionary<Keys, List<Keybinding>> _keybindingsByTriggerKey = new Dictionary<Keys, List<Keybinding>>();

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

    public void AddGlobalKeybinding(string keybindingString, Action callback)
    {
      var keybindingParts = keybindingString
        .Split('+')
        .Select(key => FormatKeybinding(key))
        .Select(key => Enum.Parse(typeof(Keys), key))
        .Cast<Keys>()
        .ToList();

      var triggerKey = keybindingParts.Last();
      var keybinding = new Keybinding(keybindingParts, callback);

      if (_keybindingsByTriggerKey.ContainsKey(triggerKey))
      {
        _keybindingsByTriggerKey[triggerKey].Add(keybinding);
        return;
      }

      _keybindingsByTriggerKey.Add(triggerKey, new List<Keybinding>() { keybinding });
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
      var registeredKeybindings = _keybindingsByTriggerKey.GetValueOrDefault(pressedKey);

      // Forward the hook notification if no keybindings exist for the trigger key.
      if (registeredKeybindings == null)
        return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);

      Keybinding matchedKeybinding = null;
      Dictionary<Keys, bool> cachedKeyStates = new Dictionary<Keys, bool>();

      foreach (var keybinding in registeredKeybindings)
      {
        var isMatch = keybinding.KeyCombination.All(key =>
        {
          if (key == pressedKey)
            return true;

          if (cachedKeyStates.ContainsKey(key))
            return cachedKeyStates[key];

          return cachedKeyStates[key] = IsKeyDown(key);
        });

        if (!isMatch)
          continue;

        // If multiple keybindings match the user input, call the longer key combination.
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
    /// Whether the given key is down.
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
