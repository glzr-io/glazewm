using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.UserConfigs;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;
using System;
using System.Diagnostics;
using static LarsWM.Domain.Common.Services.WindowsApiFacade;
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
                Window window = new Window(hwnd);

                if (!IsWindowManageable(window))
                    return true;

                // Get monitor that encompasses most of window.
                var targetMonitor = _monitorService.GetMonitorFromUnaddedWindow(window);

                _bus.Invoke(new AttachContainerCommand(targetMonitor.DisplayedWorkspace, window));

                return true;
            }, IntPtr.Zero);

            _bus.Invoke(new RedrawContainersCommand());

            return CommandResponse.Ok;
        }

        private bool IsWindowManageable(Window window)
        {
            var isApplicationWindow = IsWindowVisible(window.Hwnd)
                && !window.HasWindowStyle(WS.WS_CHILD) && !window.HasWindowExStyle(WS_EX.WS_EX_NOACTIVATE);

            var isCurrentProcess = window.Process.Id == Process.GetCurrentProcess().Id;

            var isExcludedClassName = _userConfigService.UserConfig.WindowClassesToIgnore.Contains(window.ClassName);
            var isExcludedProcessName = _userConfigService.UserConfig.ProcessNamesToIgnore.Contains(window.Process.ProcessName);

            var isShellWindow = window.Hwnd == GetShellWindow();

            if (isApplicationWindow && !isCurrentProcess && !isExcludedClassName && !isExcludedProcessName && !isShellWindow)
            {
                return true;
            }

            return false;
        }
    }
}
