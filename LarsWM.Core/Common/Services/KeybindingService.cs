using static LarsWM.Core.WindowsApi.WindowsApiService;
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Text;
using System.Reactive;
using System.Threading;
using System.Windows.Forms;
using System.Runtime.InteropServices;
using System.Reactive.Subjects;
using System.Windows.Input;
using System.Linq;
using System.ComponentModel;

namespace LarsWM.Core.Common.Services
{
    class KeybindingService
    {
        private Subject<string> _modKeypresses = new Subject<string>();

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
            //if (nCode == 0 && ((uint)wParam == 0x100 || (uint)wParam == 0x104))
            if (nCode == 0)
            {
                //var pressedKeys = inputKeys.Where(key => Keyboard.GetKeyState(key));
                // Get structure with details about keyboard input event.
                //var hookStruct = (KbLLHookStruct)Marshal.PtrToStructure(lParam, typeof(KbLLHookStruct));
                var bloop = (GetKeyState(Keys.LShiftKey) & 0x8000) == 0x8000;
                Debug.WriteIf(bloop, "bloop");
                var bleep = GetKeyState(Keys.S) < 0;
                Debug.WriteIf(bleep, "bleep");
                //var key = hookStruct.vkCode;
                IsKeyUp();
                var x = GetDownKeys().ToList();
                foreach (var k in x)
                {
                    Debug.WriteLine(k);
                }

                Debug.WriteIf(x.Count() > 0, "count");
                var ay = AnyKeyPressed();
                Debug.WriteIf(ay, "any key pressed");
                //if (key == Keys.LWin)
                //    _modKeypresses.OnNext("");
            }

            // Pass hook notification to other applications.
            return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);
        }

        private void HandleKeybindings()
        {
            _modKeypresses.Subscribe();
        }
        private static byte[] GetKeyboardState()
        {
            byte[] keyStates = new byte[256];
            if (!GetKeyboardState(keyStates))
                throw new Win32Exception(Marshal.GetLastWin32Error());
            return keyStates;
        }

        private static bool AnyKeyPressed()
        {
            byte[] keyState = GetKeyboardState();
            // skip the mouse buttons
            return keyState.Skip(8).Any(state => (state & 0x8000) != 0);
        }
        private void IsKeyUp()
        {
            byte[] keys = new byte[256];

            GetKeyboardState(keys);

            if ((keys[(int)Keys.Up] & keys[(int)Keys.Right] & 128) == 128)
            {
                Debug.WriteLine("Up Arrow key and Right Arrow key down.");
            }
        }

        private static readonly byte[] DistinctVirtualKeys = Enumerable
            .Range(0, 256)
            .Select(KeyInterop.KeyFromVirtualKey)
            .Where(item => item != Key.None)
            .Distinct()
            .Select(item => (byte)KeyInterop.VirtualKeyFromKey(item))
            .ToArray();

        /// <summary>
        /// Gets all keys that are currently in the down state.
        /// </summary>
        /// <returns>
        /// A collection of all keys that are currently in the down state.
        /// </returns>
        public IEnumerable<Key> GetDownKeys()
        {
            var keyboardState = new byte[256];
            GetKeyboardState(keyboardState);

            var downKeys = new List<Key>();
            for (var index = 0; index < DistinctVirtualKeys.Length; index++)
            {
                var virtualKey = DistinctVirtualKeys[index];
                if ((keyboardState[virtualKey] & 0x80) != 0)
                {
                    downKeys.Add(KeyInterop.KeyFromVirtualKey(virtualKey));
                }
            }

            return downKeys;
        }

        [DllImport("user32.dll")]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool GetKeyboardState(byte[] keyState);
    }
}
