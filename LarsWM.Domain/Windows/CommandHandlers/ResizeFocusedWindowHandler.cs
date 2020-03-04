using System;
using LarsWM.Domain.UserConfigs;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Windows.CommandHandlers
{
    class ResizeFocusedWindowHandler : ICommandHandler<ResizeFocusedWindowCommand>
    {
        private WindowService _windowService;
        private UserConfigService _userConfigService;

        public ResizeFocusedWindowHandler(WindowService windowService, UserConfigService userConfigService)
        {
            _windowService = windowService;
            _userConfigService = userConfigService;
        }

        public dynamic Handle(ResizeFocusedWindowCommand command)
        {
            var resizePercentage = _userConfigService.UserConfig.ResizePercentage;
            var focusedWindow = _windowService.FocusedWindow;
            var parent = focusedWindow.Parent;

            throw new NotImplementedException();
        }
    }
}
