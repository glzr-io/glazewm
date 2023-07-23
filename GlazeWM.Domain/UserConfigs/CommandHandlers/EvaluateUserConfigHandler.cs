using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text.Json;
using System.Windows.Forms;
using GlazeWM.Domain.UserConfigs.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Exceptions;
using GlazeWM.Infrastructure.Serialization;
using GlazeWM.Infrastructure.Utils;

namespace GlazeWM.Domain.UserConfigs.CommandHandlers
{
  internal sealed class EvaluateUserConfigHandler : ICommandHandler<EvaluateUserConfigCommand>
  {
    private readonly Bus _bus;
    private readonly UserConfigService _userConfigService;
    private readonly CommandParsingService _commandParsingService;

    /// <summary>
    /// Keywords that cannot be used as workspace names.
    /// </summary>
    private static readonly HashSet<string> RESERVED_WORKSPACE_NAMES = new()
    {
      "prev",
      "next",
      "recent"
    };

    public EvaluateUserConfigHandler(
      Bus bus,
      UserConfigService userConfigService,
      CommandParsingService commandParsingService)
    {
      _bus = bus;
      _userConfigService = userConfigService;
      _commandParsingService = commandParsingService;
    }

    public CommandResponse Handle(EvaluateUserConfigCommand command)
    {
      var userConfigPath = _userConfigService.UserConfigPath;

      if (!File.Exists(userConfigPath))
        InitializeSampleUserConfig(userConfigPath);

      var deserializedConfig = DeserializeUserConfig(userConfigPath);

      // Merge default window rules with user-defined rules.
      var defaultWindowRules = _userConfigService.DefaultWindowRules;
      deserializedConfig.WindowRules.InsertRange(0, defaultWindowRules);

      _userConfigService.UserConfig = deserializedConfig;

      // Check for required properties and validate command strings.
      ValidateDeserializedConfig(deserializedConfig);

      // Register keybindings.
      _bus.Invoke(new RegisterKeybindingsCommand(deserializedConfig.Keybindings));

      return CommandResponse.Ok;
    }

    private static void InitializeSampleUserConfig(string userConfigPath)
    {
      // Fix any inconsistencies in directory delimiters.
      var normalizedUserConfigPath = new FileInfo(userConfigPath).FullName;

      var promptResult = MessageBox.Show(
        $"No config file found at {normalizedUserConfigPath}. Create a new config file from the starter template?",
        "No config file found",
        MessageBoxButtons.OKCancel
      );

      if (promptResult == DialogResult.Cancel)
        throw new FatalUserException("Cannot start the app without a configuration file.");

      var assembly = Assembly.GetEntryAssembly();
      const string sampleConfigResourceName = "GlazeWM.Application.Resources.sample-config.yaml";

      // Create containing directory. Needs to be created before writing to the file.
      Directory.CreateDirectory(Path.GetDirectoryName(userConfigPath));

      // Get the embedded sample user config from the entry assembly.
      using var stream = assembly.GetManifestResourceStream(sampleConfigResourceName);

      // Write the sample user config to the appropriate destination.
      using var fileStream = new FileStream(userConfigPath, FileMode.Create, FileAccess.Write);
      stream.CopyTo(fileStream);
    }

    private static UserConfig DeserializeUserConfig(string userConfigPath)
    {
      try
      {
        var userConfigLines = File.ReadAllLines(userConfigPath);
        var input = string.Join(Environment.NewLine, userConfigLines);

        var deserializeOptions = JsonParser.OptionsFactory((options) =>
          options.Converters.Add(new BarComponentConfigConverter())
        );

        return YamlParser.ToInstance<UserConfig>(input, deserializeOptions);
      }
      catch (Exception exception)
      {
        throw new FatalUserException(FormatConfigError(exception));
      }
    }

    // TODO: Might be able to remove the required checks once nullable context is enabled.
    private void ValidateDeserializedConfig(UserConfig deserializedConfig)
    {
      try
      {
        foreach (var workspaceConfig in deserializedConfig.Workspaces)
        {
          if (workspaceConfig.Name is null)
            throw new FatalUserException("Property 'name' is required in workspace config.");

          if (RESERVED_WORKSPACE_NAMES.Contains(workspaceConfig.Name))
            throw new FatalUserException(
              $"Cannot use keyword '{workspaceConfig.Name}' as workspace name."
            );
        }

        var componentConfigs = deserializedConfig.Bar.ComponentsLeft
          .Concat(deserializedConfig.Bar.ComponentsCenter)
          .Concat(deserializedConfig.Bar.ComponentsRight);

        foreach (var componentConfig in componentConfigs)
        {
          if (componentConfig.Type is null)
            throw new FatalUserException("Property 'type' is required in bar component config.");
        }

        var keybindingsConfig = deserializedConfig.Keybindings;
        var windowRulesConfig = deserializedConfig.WindowRules;

        foreach (var keybindingConfig in keybindingsConfig)
        {
          if (keybindingConfig.BindingList.Count == 0)
            throw new FatalUserException(
              "Property 'binding' or 'bindings' is required in keybinding config."
            );

          // Check that all keybindings in the config can be cast to `Keys` enum.
          foreach (var keybinding in keybindingConfig.BindingList)
          {
            try
            {
              var _ = KeybindingHelper.ParseKeybindingString(keybinding);
            }
            catch (ArgumentException exception)
            {
              throw new FatalUserException(
                $"Invalid binding '{keybinding}' in keybinding config: {exception.Message}"
              );
            }
          }

          if (keybindingConfig.CommandList.Count == 0)
            throw new FatalUserException(
              "Property 'command' or 'commands' is required in keybinding config."
            );
        }

        foreach (var windowRuleConfig in windowRulesConfig)
        {
          var hasMatchingRegex =
            windowRuleConfig.MatchClassName is not null ||
            windowRuleConfig.MatchProcessName is not null ||
            windowRuleConfig.MatchTitle is not null;

          if (!hasMatchingRegex)
            throw new FatalUserException(
              "At least 1 matching regex (eg. 'match_process_name') is required in window rule config."
            );

          if (windowRuleConfig.CommandList.Count == 0)
            throw new FatalUserException(
              "Property 'command' or 'commands' is required in window rule config."
            );
        }

        var textComponentCommands = componentConfigs
          .OfType<TextComponentConfig>()
          .SelectMany(
            (componentConfig) => new List<string> {
              componentConfig.LeftClickCommand,
              componentConfig.RightClickCommand
            }
          )
          .Where((commandString) => !string.IsNullOrEmpty(commandString));

        var allCommandStrings = new List<string>()
          .Concat(keybindingsConfig.SelectMany((keybinding) => keybinding.CommandList))
          .Concat(windowRulesConfig.SelectMany((windowRule) => windowRule.CommandList))
          .Concat(textComponentCommands)
          .Select((commandString) => CommandParsingService.FormatCommand(commandString));

        foreach (var commandString in allCommandStrings)
          _commandParsingService.ValidateCommand(commandString);
      }
      catch (Exception exception)
      {
        throw new FatalUserException(FormatConfigError(exception));
      }
    }

    private static string FormatConfigError(Exception exception)
    {
      var errorMessage = "Failed to parse user config. ";

      // Add path to property for deserialization error messages.
      if ((exception as JsonException)?.Path is not null)
      {
        errorMessage += $"Invalid value at property: '{(exception as JsonException).Path}'.";
        return errorMessage;
      }

      errorMessage += exception.Message;
      return errorMessage;
    }
  }
}
