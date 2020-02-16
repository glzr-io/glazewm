using LarsWM.Infrastructure.WindowsApi.Enums;
using System;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Infrastructure.WindowsApi
{
    class WindowEventService
    {
        public void Initialise()
        {

            var x = new WindowEventProc();
            SetWinEventHook(EventConstant.EVENT_OBJECT_DESTROY, EventConstant.EVENT_OBJECT_SHOW, IntPtr.Zero, _hookDelegate, 0, 0, 0);
            SetWinEventHook(EventConstant.EVENT_OBJECT_CLOAKED, EventConstant.EVENT_OBJECT_UNCLOAKED, IntPtr.Zero, _hookDelegate, 0, 0, 0);
            SetWinEventHook(EventConstant.EVENT_SYSTEM_MINIMIZESTART, EventConstant.EVENT_SYSTEM_MINIMIZEEND, IntPtr.Zero, _hookDelegate, 0, 0, 0);
            SetWinEventHook(EventConstant.EVENT_SYSTEM_FOREGROUND, EventConstant.EVENT_SYSTEM_FOREGROUND, IntPtr.Zero, _hookDelegate, 0, 0, 0);
        }

        public void RegisterEventHook()
        {

        }
    }
}
