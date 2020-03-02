using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.UserConfigs;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;
using System;
using System.Diagnostics;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Domain.Windows.CommandHandlers
{
    class AddInitialWindowsHandler : ICommandHandler<AddInitialWindowsCommand>
    {
        private IBus _bus;
        private UserConfigService _userConfigService;
        private MonitorService _monitorService;
        private WindowService _windowService;

        public AddInitialWindowsHandler(
            IBus bus,
            UserConfigService userConfigService,
            MonitorService monitorService,
            WindowService windowService)
        {
            _bus = bus;
            _userConfigService = userConfigService;
            _monitorService = monitorService;
            _windowService = windowService;
        }

        public dynamic Handle(AddInitialWindowsCommand command)
        {
            EnumWindows((IntPtr hwnd, int lParam) =>
            {
                var window = new Window(hwnd);

                if (!_windowService.IsWindowManageable(window) || !window.CanLayout)
                    return true;

                // Get monitor that encompasses most of window.
                var targetMonitor = _monitorService.GetMonitorFromUnaddedWindow(window);

                // Set initial location values.
                var windowLocation = _windowService.GetLocationOfHandle(hwnd);
                window.X = windowLocation.Left;
                window.Y = windowLocation.Top;
                window.Width = windowLocation.Right - windowLocation.Left;
                window.Height = windowLocation.Bottom - windowLocation.Top;

                _bus.Invoke(new AttachContainerCommand(targetMonitor.DisplayedWorkspace, window));

                return true;
            }, IntPtr.Zero);

            _bus.Invoke(new RedrawContainersCommand());

            return CommandResponse.Ok;
        }
    }
}
