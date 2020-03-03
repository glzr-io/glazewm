using LarsWM.Domain.Common.Services;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.Monitors.Commands;
using LarsWM.Domain.UserConfigs.Commands;
using LarsWM.Domain.Windows;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;
using LarsWM.Infrastructure.WindowsApi;
using System;
using System.Linq;
using System.Windows.Forms;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Bootstrapper
{
    class Startup
    {
        private IBus _bus;
        private MonitorService _monitorService;
        private KeybindingService _keybindingService;
        private WindowEventService _windowEventService;
        private WindowHooksHandler _windowHooksHandler;
        private WindowService _windowService;

        public Startup(
            IBus bus,
            MonitorService monitorService,
            KeybindingService keybindingService,
            WindowEventService windowEventService,
            WindowHooksHandler windowHooksHandler,
            WindowService windowService)
        {
            _bus = bus;
            _monitorService = monitorService;
            _keybindingService = keybindingService;
            _windowEventService = windowEventService;
            _windowHooksHandler = windowHooksHandler;
            _windowService = windowService;
        }

        public void Init()
        {
            // Populate initial monitors, windows, workspaces and user config.
            PopulateInitialState();

            _keybindingService.Init();

            _windowEventService.Init();
            _windowHooksHandler.Configure();
        }

        /// <summary>
        /// Populate initial monitors, windows, workspaces and user config.
        /// </summary>
        private void PopulateInitialState()
        {
            // Read user config file and set its values in state.
            _bus.Invoke(new EvaluateUserConfigCommand());

            // Create a Monitor and consequently a Workspace for each detected Screen.
            foreach (var screen in Screen.AllScreens)
                _bus.Invoke(new AddMonitorCommand(screen));

            // Add initial windows to tree.
            _bus.Invoke(new AddInitialWindowsCommand());

            var focusedWindow = _windowService.GetWindows().FirstOrDefault(w => w.Hwnd == GetForegroundWindow());

            // GetForegroundWindow might return a handle that is not in tree. Return first window in such cases.
            if (focusedWindow == null)
                focusedWindow = _windowService.GetWindows().First();

            // Set currently focused window.
            _bus.Invoke(new FocusWindowCommand(focusedWindow));
        }
    }
}

