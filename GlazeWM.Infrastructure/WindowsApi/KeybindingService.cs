using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Runtime.InteropServices;
using System.Threading;
using System.Windows.Forms;
using GlazeWM.Infrastructure.Bussing;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Infrastructure.WindowsApi
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
    private const uint WM_KEYDOWN = 0x100;
    private const uint WM_SYSKEYDOWN = 0x104;

    /// <summary>
    /// Registered keybindings grouped by trigger key (ie. the final key in a key combination).
    /// </summary>
    private readonly Dictionary<Keys, List<Keybinding>> _keybindingsByTriggerKey
      = new Dictionary<Keys, List<Keybinding>>();

    public void Start()
    {
      var thread = new Thread(() => CreateKeybindingHook())
      {
        Name = "GlazeWMKeybindingService"
      };
      thread.Start();
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

    private static string FormatKeybinding(string key)
    {
      var isNumeric = int.TryParse(key, out var _);

      return isNumeric ? $"D{key}" : key;
    }

    private void CreateKeybindingHook()
    {
      _ = SetWindowsHookEx(HookType.WH_KEYBOARD_LL, KeybindingHookProc, Process.GetCurrentProcess().MainModule.BaseAddress, 0);

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

      var cachedKeyStates = new Dictionary<Keys, bool>();

      var matchedKeybindings = registeredKeybindings.Where(keybinding =>
      {
        return keybinding.KeyCombination.All(key =>
        {
          if (key == pressedKey)
            return true;

          if (cachedKeyStates.ContainsKey(key))
            return cachedKeyStates[key];

          return cachedKeyStates[key] = IsKeyDown(key);
        });
      });

      // If multiple keybindings match the user input, call the longest key combination.
      var longestKeybinding = matchedKeybindings
        .OrderByDescending(keybinding => keybinding.KeyCombination.Count)
        .FirstOrDefault();

      if (longestKeybinding == null)
        return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);

      // Invoke the matched keybinding.
      longestKeybinding.KeybindingProc();

      // Avoid forwarding the key input to other applications.
      return new IntPtr(1);
    }

    /// <summary>
    /// Get whether the given key is down. Check alternate versions of modifier keys.
    /// </summary>
    private static bool IsKeyDown(Keys key)
    {
      if (key == Keys.Alt)
        return IsKeyDownRaw(Keys.LMenu) || IsKeyDownRaw(Keys.RMenu);

      if (key == Keys.Shift)
        return IsKeyDownRaw(Keys.LShiftKey) || IsKeyDownRaw(Keys.RShiftKey);

      if (key == Keys.Control)
        return IsKeyDownRaw(Keys.LControlKey) || IsKeyDownRaw(Keys.RControlKey);

      return IsKeyDownRaw(key);
    }

    /// <summary>
    /// Get whether the given key is down. If the high-order bit is 1, the key is down; otherwise, it is up.
    /// </summary>
    private static bool IsKeyDownRaw(Keys key)
    {
      return (GetKeyState(key) & 0x8000) == 0x8000;
    }
  }
}
