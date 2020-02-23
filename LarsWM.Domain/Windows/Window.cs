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

        public bool HasWindowStyle(WS style)
        {
            return _windowService.HandleHasWindowStyle(Hwnd, style);
        }

        public bool HasWindowExStyle(WS_EX style)
        {
            return _windowService.HandleHasWindowExStyle(Hwnd, style);
        }
    }
}
