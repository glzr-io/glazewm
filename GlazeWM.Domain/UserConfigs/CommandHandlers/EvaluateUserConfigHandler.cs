using GlazeWM.Domain.UserConfigs.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Exceptions;
using GlazeWM.Infrastructure.Yaml;
using Newtonsoft.Json;
using System;
using System.IO;
using System.Reflection;
using System.Text.RegularExpressions;
using System.Windows.Forms;

namespace GlazeWM.Domain.UserConfigs.CommandHandlers
{
  internal class EvaluateUserConfigHandler : ICommandHandler<EvaluateUserConfigCommand>
  {
    private readonly Bus _bus;
    private readonly UserConfigService _userConfigService;
    private readonly YamlDeserializationService _yamlDeserializationService;

    public EvaluateUserConfigHandler(
      Bus bus,
      UserConfigService userConfigService,
      YamlDeserializationService yamlDeserializationService
    )
    {
      _bus = bus;
      _userConfigService = userConfigService;
      _yamlDeserializationService = yamlDeserializationService;
    }

    public CommandResponse Handle(EvaluateUserConfigCommand command)
    {
      UserConfig deserializedConfig;

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

      // Merge default window rules with user-defined rules.
      var defaultWindowRules = _userConfigService.DefaultWindowRules;
      deserializedConfig.WindowRules.InsertRange(0, defaultWindowRules);

      _userConfigService.UserConfig = deserializedConfig;

      // Register keybindings.
      _bus.Invoke(new RegisterKeybindingsCommand(deserializedConfig.Keybindings));

      return CommandResponse.Ok;
    }

    private static void InitializeSampleUserConfig(string userConfigPath)
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
      const string sampleConfigResourceName = "GlazeWM.Bootstrapper.sample-config.yaml";

      // Create containing directory. Needs to be created before writing to the file.
      Directory.CreateDirectory(Path.GetDirectoryName(userConfigPath));

      // Get the embedded sample user config from the entry assembly.
      using var stream = assembly.GetManifestResourceStream(sampleConfigResourceName);

      // Write the sample user config to the appropriate destination.
      using var fileStream = new FileStream(userConfigPath, FileMode.Create, FileAccess.Write);
      stream.CopyTo(fileStream);
    }

    private UserConfig DeserializeUserConfig(string userConfigPath)
    {
      var userConfigLines = File.ReadAllLines(userConfigPath);
      var input = new StringReader(string.Join(Environment.NewLine, userConfigLines));

      return _yamlDeserializationService.Deserialize<UserConfig>(input);
    }

    private static string FormatErrorMessage(Exception exception)
    {
      var errorMessage = "Failed to parse user config. ";

      var unknownPropertyRegex = new Regex("Could not find member '(?<property>.*?)' on object");
      var unknownPropertyMatch = unknownPropertyRegex.Match(exception.Message);

      // Improve error message in case of unknown property errors.
      if (unknownPropertyMatch.Success)
        errorMessage += $"Unknown property: '{unknownPropertyMatch.Groups["property"]}'.";

      // Improve error message of generic deserialization errors.
      else if (exception is JsonReaderException)
        errorMessage += $"Invalid value at property: '{(exception as JsonReaderException).Path}'.";
      else
        errorMessage += exception.Message;

      return errorMessage;
    }
  }
}
