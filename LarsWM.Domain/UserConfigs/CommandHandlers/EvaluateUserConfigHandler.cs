using LarsWM.Domain.UserConfigs.Commands;
using LarsWM.Domain.Workspaces.Commands;
using LarsWM.Infrastructure.Bussing;
using LarsWM.Infrastructure.WindowsApi;
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
    private KeybindingService _keybindingService;

    public EvaluateUserConfigHandler(Bus bus, UserConfigService userConfigService, KeybindingService keybindingService)
    {
      _bus = bus;
      _userConfigService = userConfigService;
      _keybindingService = keybindingService;
    }

    public CommandResponse Handle(EvaluateUserConfigCommand command)
    {
      UserConfig deserializedConfig = null;

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

      _keybindingService.SetModKey(deserializedConfig.ModKey);
      _bus.Invoke(new RegisterKeybindingsCommand(deserializedConfig.Keybindings));

      _userConfigService.UserConfig = deserializedConfig;

      return CommandResponse.Ok;
    }

    private UserConfig DeserializeUserConfig(string userConfigPath)
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

      return deserializer.Deserialize<UserConfig>(input);
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
