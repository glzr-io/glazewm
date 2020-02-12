using LarsWM.Domain.Common.Models;
using LarsWM.Domain.UserConfigs.Commands;

namespace LarsWM.Domain.UserConfigs.CommandHandlers
{
    class ReadUserConfigHandler : ICommandHandler<ReadUserConfigCommand>
    {
        private UserConfigService _userConfigService;

        public ReadUserConfigHandler(UserConfigService userConfigService)
        {
            _userConfigService = userConfigService;
        }

        public void Handle(ReadUserConfigCommand command)
        {
            // TODO: Read user config from file / constructed through shell script.
            var userConfig = new UserConfig();
            _userConfigService.UserConfig = userConfig;
        }
    }
}
