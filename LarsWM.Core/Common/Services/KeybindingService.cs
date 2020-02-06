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
            if (nCode < 0)
                return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);

            // Get structure with details about keyboard input event.
            var hookStruct = (KbLLHookStruct) Marshal.PtrToStructure(lParam, typeof(KbLLHookStruct));

            var key = hookStruct.vkCode;

            if (key == Keys.LWin)
                _modKeypresses.OnNext("");

            // Pass hook notification to other applications.
            return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);
        }

        private void HandleKeybindings()
        {
            _modKeypresses.Subscribe();
        }
    }
}
