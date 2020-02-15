using LarsWM.Domain.UserConfigs.Commands;
using LarsWM.Infrastructure.Bussing;
using System;
using System.IO;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace LarsWM.Domain.UserConfigs.CommandHandlers
{
    class ReadUserConfigHandler : ICommandHandler<ReadUserConfigCommand>
    {
        private UserConfigService _userConfigService;

        public ReadUserConfigHandler(UserConfigService userConfigService)
        {
            _userConfigService = userConfigService;
        }

        public CommandResponse Handle(ReadUserConfigCommand command)
        {
            var userConfigPath = Path.Combine(Directory.GetCurrentDirectory(), "UserConfigs/SampleUserConfig.yaml");

            var userConfigLines = File.ReadAllLines(userConfigPath);
            var input = new StringReader(string.Join(Environment.NewLine, userConfigLines));

            var deserializer = new DeserializerBuilder()
                .WithNamingConvention(PascalCaseNamingConvention.Instance)
                .Build();

            var deserializedConfig = deserializer.Deserialize<UserConfigFileDto>(input);

            // TODO: Read user config from file / constructed through shell script.
            var userConfig = new UserConfig();
            _userConfigService.UserConfig = userConfig;

            return CommandResponse.Ok;
        }
    }
}
