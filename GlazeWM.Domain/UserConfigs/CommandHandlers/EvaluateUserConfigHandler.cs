using GlazeWM.Domain.UserConfigs.Commands;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi;
using GlazeWM.Infrastructure.Yaml;
using System;
using System.IO;
using System.Reflection;
using System.Text.RegularExpressions;
using System.Windows.Forms;

namespace GlazeWM.Domain.UserConfigs.CommandHandlers
{
  class EvaluateUserConfigHandler : ICommandHandler<EvaluateUserConfigCommand>
  {
    private Bus _bus;
    private UserConfigService _userConfigService;
    private KeybindingService _keybindingService;
    private YamlDeserializationService _yamlDeserializationService;

    public EvaluateUserConfigHandler(Bus bus, UserConfigService userConfigService, KeybindingService keybindingService, YamlDeserializationService yamlDeserializationService)
    {
      _bus = bus;
      _userConfigService = userConfigService;
      _keybindingService = keybindingService;
      _yamlDeserializationService = yamlDeserializationService;
    }

    public CommandResponse Handle(EvaluateUserConfigCommand command)
    {
      UserConfig deserializedConfig = null;

      try
      {
        var userConfigPath = _userConfigService.UserConfigPath;

        if (!File.Exists(userConfigPath))
          InitializeSampleUserConfig(userConfigPath);

        deserializedConfig = DeserializeUserConfig(userConfigPath);
      }
      catch (Exception exception)
      {
        var errorMessage = FormatErrorMessage(exception);
        throw new FatalUserException(errorMessage);
      }

      // Register keybindings.
      _bus.Invoke(new RegisterKeybindingsCommand(deserializedConfig.Keybindings));

      // Merge default window rules with user-defined rules.
      var defaultWindowRules = _userConfigService.DefaultWindowRules;
      deserializedConfig.WindowRules.InsertRange(0, defaultWindowRules);

      _userConfigService.UserConfig = deserializedConfig;

      return CommandResponse.Ok;
    }

    private void InitializeSampleUserConfig(string userConfigPath)
    {
      // Fix any inconsistencies in directory delimiters.
      var normalizedUserConfigPath = Path.GetFullPath(new Uri(userConfigPath).LocalPath);

      var promptResult = MessageBox.Show(
        $"No config file found at {normalizedUserConfigPath}. Create a new config file from the starter template?",
        "No config file found",
        MessageBoxButtons.OKCancel
      );

      if (promptResult == DialogResult.Cancel)
        throw new FatalUserException("Cannot start the app without a configuration file.");

      var assembly = Assembly.GetEntryAssembly();
      var sampleConfigResourceName = "GlazeWM.Bootstrapper.sample-config.yaml";

      // Get the embedded sample user config from the entry assembly.
      using (Stream stream = assembly.GetManifestResourceStream(sampleConfigResourceName))
      {
        // Write the sample user config to the appropriate destination.
        using (var fileStream = new FileStream(userConfigPath, FileMode.Create, FileAccess.Write))
        {
          Directory.CreateDirectory(Path.GetDirectoryName(userConfigPath));
          stream.CopyTo(fileStream);
        }
      }
    }

    private UserConfig DeserializeUserConfig(string userConfigPath)
    {
      var userConfigLines = File.ReadAllLines(userConfigPath);
      var input = new StringReader(string.Join(Environment.NewLine, userConfigLines));

      return _yamlDeserializationService.Deserialize<UserConfig>(input);
    }

    private string FormatErrorMessage(Exception exception)
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

      return errorMessage;
    }
  }
}
