using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Threading;
using System.Windows.Forms;
using Gma.System.MouseKeyHook;
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
    private Keys _modKey = Keys.Alt;
    private List<Keys> _triggerKeys = new List<Keys>();

    private Bus _bus;
    private IKeyboardMouseEvents m_GlobalHook;

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

    public void AddGlobalKeybinding(string binding, Action callback)
    {
      var bindingParts = binding
        .Split('+')
        .Select(key => FormatKeybinding(key));

      var formattedBinding = string.Join("+", bindingParts);
      var combinationBinding = Combination.FromString(formattedBinding);

      var combinationActionDic = new Dictionary<Combination, Action>
      {
        { combinationBinding, callback }
      };

      m_GlobalHook.OnCombination(combinationActionDic);
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
      m_GlobalHook = Hook.GlobalEvents();

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
