using LarsWM.Domain.UserConfigs.Commands;
using LarsWM.Domain.Workspaces.Commands;
using LarsWM.Infrastructure.Bussing;
using System;
using System.IO;
using System.Text.RegularExpressions;
using System.Windows;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;
using YamlDotNet.Serialization.NodeDeserializers;

namespace LarsWM.Domain.UserConfigs.CommandHandlers
{
  class EvaluateUserConfigHandler : ICommandHandler<EvaluateUserConfigCommand>
  {
    private UserConfigService _userConfigService;
    private Bus _bus;

    public EvaluateUserConfigHandler(UserConfigService userConfigService, Bus bus)
    {
      _userConfigService = userConfigService;
      _bus = bus;
    }

    public dynamic Handle(EvaluateUserConfigCommand command)
    {
      var userfolder = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
      var userConfigPath = Path.Combine(userfolder, "./.glaze-wm/config.yaml");

      var userConfigLines = File.ReadAllLines(userConfigPath);
      var input = new StringReader(string.Join(Environment.NewLine, userConfigLines));

      var deserializer = new DeserializerBuilder()
        .WithNamingConvention(PascalCaseNamingConvention.Instance)
        .WithNodeDeserializer(inner => new ValidatingDeserializer(inner), s => s.InsteadOf<ObjectNodeDeserializer>())
        .Build();

      UserConfigFileDto deserializedConfig = new UserConfigFileDto();

      try
      {
        deserializedConfig = deserializer.Deserialize<UserConfigFileDto>(input);
      }
      catch (Exception exception)
      {
        var errorMessage = exception.Message;

        if (exception.InnerException?.Message != null)
        {
          var unknownPropertyRegex = new Regex(@"Property '(?<property>.*?)' not found on type");
          var match = unknownPropertyRegex.Match(exception.InnerException.Message);

          // Improve error message shown in case of unknown property error.
          if (match.Success)
            errorMessage = $"Unknown property in config: {match.Groups["property"]}.";
          else
            errorMessage += $". {exception.InnerException.Message}";
        }

        // Alert the user of the config error.
        MessageBox.Show(errorMessage);

        throw exception;
      }

      foreach (var workspaceConfig in deserializedConfig.Workspaces)
        _bus.Invoke(new CreateWorkspaceCommand(workspaceConfig.Name));

      // TODO: Read user config from file / constructed through shell script.
      var userConfig = new UserConfig();
      _userConfigService.UserConfig = userConfig;

      return CommandResponse.Ok;
    }
  }
}
