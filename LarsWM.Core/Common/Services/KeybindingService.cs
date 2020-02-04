using static LarsWM.Core.WindowsApi.WindowsApiService;
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Text;
using System.Threading;
using System.Windows.Forms;

namespace LarsWM.Core.Common.Services
{
    class KeybindingService
    {
        public void Init()
        {

            // SetWindowsHookEx requires a message loop within the thread that is executing the code.
            var thread = new Thread(() =>
            {
                SetWindowsHookEx(WH_KEYBOARD_LL, _kbdHook, Process.GetCurrentProcess().MainModule.BaseAddress, 0);
                SetWindowsHookEx(WH_MOUSE_LL, _mouseHook, Process.GetCurrentProcess().MainModule.BaseAddress, 0);
                Application.Run();
            });
            thread.Name = "LarsWMKeybindingService";
            thread.Start();
        }

        private int KbHookProc(int nCode, IntPtr wParam, IntPtr lParam)
        {
            if (nCode >= 0) // This means we can intercept the event.
            {
                var hookStruct = (KbLLHookStruct)Marshal.PtrToStructure(
                        lParam,
                        typeof(KbLLHookStruct));

                // Quick check if Ctrl key is down. 
                // See GetKeyState() doco for more info about the flags.
                bool ctrlDown =
                        GetKeyState(VK_LCONTROL) != 0 ||
                        GetKeyState(VK_RCONTROL) != 0;

                if (ctrlDown && hookStruct.vkCode == 0x56) // Ctrl+V
                {
                    // Replace this with your custom action.
                    Clipboard.SetText("Hi");
                }
            }

            // Pass to other keyboard handlers. Makes the Ctrl+V pass through.
            return CallNextHookEx(_hookHandle, nCode, wParam, lParam);
        }
    }
}
