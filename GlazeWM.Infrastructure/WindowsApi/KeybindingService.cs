using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Runtime.InteropServices;
using System.Windows.Forms;
using GlazeWM.Infrastructure.Utils;
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
    /// Hwnd of currently focused window and it's known children
    /// </summary>
    private List<IntPtr> FocusedWindows = new List<IntPtr>();

    /// <summary>
    /// Registered keybindings grouped by trigger key (ie. the final key in a key combination).
    /// </summary>
    private readonly Dictionary<Keys, List<Keybinding>> _keybindingsByTriggerKey = new();

    private readonly Keys[] _modifierKeys = new Keys[] {
      Keys.LControlKey,
      Keys.RControlKey,
      Keys.LMenu,
      Keys.RMenu,
      Keys.LShiftKey,
      Keys.RShiftKey
    };

    /// <summary>
    /// Store a reference to the hook delegate to prevent its garbage collection.
    /// </summary>
    private readonly HookProc _hookProc;
    private readonly HookProc _hookProcMouse;

    public KeybindingService()
    {
      _hookProc = new HookProc(KeybindingHookProc);
      _hookProcMouse = new HookProc(MouseHookProc);
    }

    public void Start()
    {
      // Create low-level keyboard hook.
      _ = SetWindowsHookEx(
        HookType.KeyboardLowLevel,
        _hookProc,
        Process.GetCurrentProcess().MainModule.BaseAddress,
        0
      );
      // Create low-level mouse hook.
      _ = SetWindowsHookEx(
        HookType.WH_MOUSE_LL,
        _hookProcMouse,
        Process.GetCurrentProcess().MainModule.BaseAddress,
        0
      );
    }

    public void AddGlobalKeybinding(string keybindingString, Action callback)
    {
      var keybindingKeys = KeybindingHelper.ParseKeybindingString(keybindingString);

      var triggerKey = keybindingKeys.Last();
      var keybinding = new Keybinding(keybindingKeys, callback);

      if (_keybindingsByTriggerKey.ContainsKey(triggerKey))
      {
        _keybindingsByTriggerKey[triggerKey].Add(keybinding);
        return;
      }

      _keybindingsByTriggerKey.Add(triggerKey, new List<Keybinding>() { keybinding });
    }

    public void Reset()
    {
      _keybindingsByTriggerKey.Clear();
    }

    private IntPtr MouseHookProc(int nCode, IntPtr wParam, IntPtr lParam)
    {
      var inputEvent = (LowLevelMouseInputEvent)Marshal.PtrToStructure(
        lParam,
        typeof(LowLevelMouseInputEvent)
      );

      // Returns window underneath cursor.  This could be a child window or parent.
      IntPtr window = WindowFromPoint(inputEvent.pt);

      // If the mouse is hovering over the currently focused main window or one of it's children, do nothing.
      if (FocusedWindows.Contains(window))
        return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);

      // If the FocusedWindows list didn't contain the window, this must be a new window being focused.
      FocusedWindows.Clear();
      FocusedWindows.Add(window);

      // Check if the window is the main window or a child window.
      IntPtr parentWindow = GetParent(window);

      // Walk the window up each parent window until you have the main window.
      while (parentWindow != IntPtr.Zero)
      {
        window = parentWindow;
        FocusedWindows.Add(window);
        parentWindow = GetParent(window);
      }

      // Focus the main window
      SetForegroundWindow(window);
      SetFocus(window);

      return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);
    }

    private IntPtr KeybindingHookProc(int nCode, IntPtr wParam, IntPtr lParam)
    {
      var shouldPassThrough =
        nCode != 0 || !((uint)wParam == WM_KEYDOWN || (uint)wParam == WM_SYSKEYDOWN);

      // If nCode is less than zero, the hook procedure must pass the hook notification to other
      // applications via `CallNextHookEx`.
      if (shouldPassThrough)
        return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);

      // Get struct with details about keyboard input event.
      var inputEvent = (LowLevelKeyboardInputEvent)Marshal.PtrToStructure(
        lParam,
        typeof(LowLevelKeyboardInputEvent)
      );

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

      // Get modifier keys that aren't part of the key combination.
      var modifierKeysToReject = _modifierKeys.Where(
        (modifierKey) =>
          !longestKeybinding.KeyCombination.Contains(modifierKey) &&
          !longestKeybinding.KeyCombination.Contains(GetGenericKey(modifierKey))
      );

      var hasModifierKeysToReject = modifierKeysToReject.Any((modifierKey) =>
        cachedKeyStates.ContainsKey(modifierKey)
          ? cachedKeyStates[modifierKey]
          : IsKeyDown(modifierKey)
      );

      // This makes sure that if you press Control+Alt+1, and you have a keybinding of Alt+1, that
      // it doesn't fire. Even though that all the keys satisfy it, the event has too many keys,
      // so it shouldn't fire.
      if (hasModifierKeysToReject)
        return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);

      // Invoke the matched keybinding.
      longestKeybinding.KeybindingProc();

      // Avoid forwarding the key input to other applications.
      return new IntPtr(1);
    }

    private static Keys GetGenericKey(Keys key)
    {
      return key switch
      {
        Keys.LMenu or Keys.RMenu => Keys.Alt,
        Keys.LShiftKey or Keys.RShiftKey => Keys.Shift,
        Keys.LControlKey or Keys.RControlKey => Keys.Control,
        _ => key,
      };
    }

    /// <summary>
    /// Get whether the given key is down. Check alternate versions of modifier keys.
    /// </summary>
    private static bool IsKeyDown(Keys key)
    {
      return key switch
      {
        Keys.Alt => IsKeyDownRaw(Keys.LMenu) || IsKeyDownRaw(Keys.RMenu),
        Keys.Shift => IsKeyDownRaw(Keys.LShiftKey) || IsKeyDownRaw(Keys.RShiftKey),
        Keys.Control => IsKeyDownRaw(Keys.LControlKey) || IsKeyDownRaw(Keys.RControlKey),
        _ => IsKeyDownRaw(key),
      };
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
