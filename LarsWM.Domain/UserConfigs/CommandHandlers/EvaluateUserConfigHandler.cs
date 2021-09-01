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
    private Bus _bus;
    private UserConfigService _userConfigService;

    public EvaluateUserConfigHandler(Bus bus, UserConfigService userConfigService)
    {
      _bus = bus;
      _userConfigService = userConfigService;
    }

    public dynamic Handle(EvaluateUserConfigCommand command)
    {
      UserConfigFileDto deserializedConfig = null;

      try
      {
        var userConfigPath = _userConfigService.UserConfigPath;

        if (!File.Exists(userConfigPath))
        {
          // Initialize the user config with the sample config.
          Directory.CreateDirectory(Path.GetDirectoryName(userConfigPath));
          File.Copy(_userConfigService.SampleUserConfigPath, userConfigPath, false);
        }

        deserializedConfig = DeserializeUserConfig(userConfigPath);
      }
      catch (Exception exception)
      {
        ShowErrorAlert(exception);
        throw exception;
      }

      // Create an inactive `Workspace` for each workspace config.
      foreach (var workspaceConfig in deserializedConfig.Workspaces)
        _bus.Invoke(new CreateWorkspaceCommand(workspaceConfig.Name));

      // TODO: Read user config from file / constructed through shell script.
      var userConfig = new UserConfig();

      _userConfigService.UserConfig = userConfig;

      return CommandResponse.Ok;
    }


    private UserConfigFileDto DeserializeUserConfig(string userConfigPath)
    {
      var userConfigLines = File.ReadAllLines(userConfigPath);
      var input = new StringReader(string.Join(Environment.NewLine, userConfigLines));

      var deserializer = new DeserializerBuilder()
        .WithNamingConvention(PascalCaseNamingConvention.Instance)
        .WithNodeDeserializer(
          inner => new ValidatingDeserializer(inner),
          component => component.InsteadOf<ObjectNodeDeserializer>()
        )
        .Build();

      return deserializer.Deserialize<UserConfigFileDto>(input);
    }

    private void ShowErrorAlert(Exception exception)
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
    }
  }
}
