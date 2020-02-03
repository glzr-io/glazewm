using LarsWM.Core.Common.Models;
using LarsWM.Core.UserConfigs.Commands;

namespace LarsWM.Core.UserConfigs.CommandHandlers
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
