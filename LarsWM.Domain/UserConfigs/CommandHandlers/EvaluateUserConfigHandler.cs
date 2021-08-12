using LarsWM.Domain.UserConfigs.Commands;
using LarsWM.Domain.Workspaces.Commands;
using LarsWM.Infrastructure.Bussing;
using System;
using System.IO;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace LarsWM.Domain.UserConfigs.CommandHandlers
{
    class EvaluateUserConfigHandler : ICommandHandler<EvaluateUserConfigCommand>
    {
        private UserConfigService _userConfigService;
        private IBus _bus;

        public EvaluateUserConfigHandler(UserConfigService userConfigService, IBus bus)
        {
            _userConfigService = userConfigService;
            _bus = bus;
        }

        public dynamic Handle(EvaluateUserConfigCommand command)
        {
            // TODO: Change user config path to be somewhere in home directory.
            var userConfigPath = Path.Combine(Directory.GetCurrentDirectory(), "../LarsWM.Domain/UserConfigs/SampleUserConfig.yaml");

            var userConfigLines = File.ReadAllLines(userConfigPath);
            var input = new StringReader(string.Join(Environment.NewLine, userConfigLines));

            var deserializer = new DeserializerBuilder()
                .WithNamingConvention(PascalCaseNamingConvention.Instance)
                .Build();

            var deserializedConfig = deserializer.Deserialize<UserConfigFileDto>(input);

            foreach (var workspaceConfig in deserializedConfig.Workspaces)
            {
                _bus.Invoke(new CreateWorkspaceCommand(workspaceConfig.Name));
            }

            // TODO: Read user config from file / constructed through shell script.
            var userConfig = new UserConfig();
            _userConfigService.UserConfig = userConfig;

            return CommandResponse.Ok;
        }
    }
}
