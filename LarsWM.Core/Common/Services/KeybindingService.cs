using LarsWM.Domain.Monitors.Events;
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Reactive.Subjects;
using System.Runtime.InteropServices;
using System.Threading;
using System.Windows.Forms;
using static LarsWM.Domain.WindowsApi.WindowsApiService;

namespace LarsWM.Domain.Common.Services
{
    class KeybindingService
    {
        private Subject<Keys> _modKeypresses = new Subject<Keys>();
        public static readonly uint WM_KEYDOWN = 0x100;
        public static readonly uint WM_SYSKEYDOWN = 0x104;

        private IBus _bus;

        public KeybindingService(IBus bus)
        {
            _bus = bus;
        }

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
            // TODO: Flatten this if-statement.
            if (nCode == 0 && ((uint)wParam == WM_KEYDOWN || (uint)wParam == WM_SYSKEYDOWN))
            {
                var modifiersPressed = new List<Keys>();

                if ((GetKeyState(Keys.LMenu) & 0x8000) == 0x8000)
                    modifiersPressed.Add(Keys.LMenu);

                if (modifiersPressed.Count() == 0)
                    return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);

                // Get structure with details about keyboard input event.
                var hookStruct = (KbLLHookStruct) Marshal.PtrToStructure(lParam, typeof(KbLLHookStruct));

                _modKeypresses.OnNext(hookStruct.vkCode);
                return new IntPtr(1);
            }

            return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);
        }

        private void HandleKeybindings()
        {
            _modKeypresses.Subscribe(pressedKey =>
            {
                if (pressedKey == Keys.J)
                    _bus.RaiseEvent(new MonitorAddedEvent());
            }
            );
        }
    }
}
