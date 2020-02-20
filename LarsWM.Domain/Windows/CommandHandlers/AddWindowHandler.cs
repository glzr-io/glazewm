using LarsWM.Domain.UserConfigs;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;
using System.Diagnostics;
using static LarsWM.Domain.Common.Services.WindowsApiFacade;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Domain.Windows.CommandHandlers
{
    class AddWindowHandler : ICommandHandler<AddWindowCommand>
    {
        private UserConfigService _userConfigService;

        public AddWindowHandler(UserConfigService userConfigService)
        {
            _userConfigService = userConfigService;
        }

        public dynamic Handle(AddWindowCommand command)
        {
            Window window = new Window(command.WindowHandle);

            if (!IsWindowManageable(window))
                return new CommandResponse(false, window.Id);

            return new CommandResponse(true, window.Id);
        }

        private bool IsWindowManageable(Window window)
        {
            var isApplicationWindow = IsWindowVisible(window.Hwnd) && !HasWindowStyle(window.Hwnd, WS.WS_CHILD) && !HasWindowExStyle(window.Hwnd, WS_EX.WS_EX_NOACTIVATE);

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
