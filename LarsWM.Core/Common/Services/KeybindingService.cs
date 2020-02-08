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
        private Subject<List<Key>> _modKeypresses = new Subject<List<Key>>();

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
            if (nCode < 0)
                return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);

            var isModKeypress = (GetKeyState(Keys.LWin) & 0x8000) == 0x8000;
            Debug.WriteLine(isModKeypress);

            if (!isModKeypress)
                return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);

            var downKeys = GetDownKeys().ToList();
            if (downKeys.Count() == 1)
                return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);

            _modKeypresses.OnNext(downKeys);

            return new IntPtr(1);
        }

        private void HandleKeybindings()
        {
            _modKeypresses.Subscribe(downKeys =>
            {
                foreach (var key in downKeys)
                {
                    Debug.WriteLine(key);
                }
            }
            );
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
