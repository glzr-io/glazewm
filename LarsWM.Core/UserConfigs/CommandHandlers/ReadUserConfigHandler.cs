using LarsWM.Core.Common.Models;
using LarsWM.Core.UserConfigs.Commands;

namespace LarsWM.Core.UserConfigs.CommandHandlers
{
    class ReadUserConfigHandler : ICommandHandler<ReadUserConfigCommand>
    {
        private AppState _appState;

        public ReadUserConfigHandler(AppState appState)
        {
            _appState = appState;
        }

        public void Handle(ReadUserConfigCommand command)
        {
            // TODO: Read user config from file / shell script.
            var userConfig = new UserConfig();
            _appState.UserConfig = userConfig;
        }
    }
}
