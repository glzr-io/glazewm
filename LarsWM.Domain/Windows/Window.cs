using LarsWM.Domain.Common.Models;
using LarsWM.Infrastructure;
using LarsWM.Infrastructure.WindowsApi;
using Microsoft.Extensions.DependencyInjection;
using System;
using System.Diagnostics;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Domain.Windows
{
    public class Window : Container
    {
        public Guid Id = Guid.NewGuid();
        public IntPtr Hwnd { get; }

        private WindowService _windowService;

        public Window(IntPtr hwnd)
        {
            Hwnd = hwnd;
            _windowService = ServiceLocator.Provider.GetRequiredService<WindowService>();
        }

        public Process Process => _windowService.GetProcessOfHandle(Hwnd);

        public string ClassName => _windowService.GetClassNameOfHandle(Hwnd);

        public WindowRect Location => _windowService.GetLocationOfHandle(Hwnd);

        public bool CanLayout => !_windowService.IsHandleCloaked(Hwnd)
            && _windowService.IsHandleManageable(Hwnd);

        public void Remove()
        {
            // Clear from list of windows in AppState
            // Clear from WindowsInWorkspace of workspaces in AppState
        }

        /// <summary>
        /// This window's style flags.
        /// </summary>
        //public WindowStyleFlags Style
        //{
        //    get
        //    {
        //        return unchecked((WindowStyleFlags)GetWindowLongPtr(_hwnd, (int)(GWL.GWL_STYLE)).ToInt64());
        //    }
        //    set
        //    {
        //        SetWindowLong(_hwnd, (int)GWL.GWL_STYLE, (int)value);
        //    }

        //}

        /// <summary>
        /// This window's extended style flags.
        /// </summary>
        //[CLSCompliant(false)]
        //public WindowExStyleFlags ExtendedStyle
        //{
        //    get
        //    {
        //        return unchecked((WindowExStyleFlags)GetWindowLongPtr(_hwnd, (int)(GWL.GWL_EXSTYLE)).ToInt64());
        //    }
        //    set
        //    {
        //        SetWindowLong(_hwnd, (int)GWL.GWL_EXSTYLE, (int)value);
        //    }
        //}
    }
}
