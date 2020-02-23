using System;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.UserConfigs;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Windows.CommandHandlers
{
    class AddWindowHandler : ICommandHandler<AddWindowCommand>
    {
        private UserConfigService _userConfigService;
        private MonitorService _monitorService;

        public AddWindowHandler(UserConfigService userConfigService, MonitorService monitorService)
        {
            _userConfigService = userConfigService;
            _monitorService = monitorService;
        }

        public dynamic Handle(AddWindowCommand command)
        {
            throw new NotImplementedException();
        }
    }
}
