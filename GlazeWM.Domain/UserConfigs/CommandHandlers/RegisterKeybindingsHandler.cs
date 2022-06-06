using System.Linq;
using GlazeWM.Domain.UserConfigs.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.UserConfigs.CommandHandlers
{
  internal class RegisterKeybindingsHandler : ICommandHandler<RegisterKeybindingsCommand>
  {
    private readonly Bus _bus;
    private readonly KeybindingService _keybindingService;
    private readonly CommandParsingService _commandParsingService;

    public RegisterKeybindingsHandler(Bus bus, KeybindingService keybindingService, CommandParsingService commandParsingService)
    {
      _bus = bus;
      _keybindingService = keybindingService;
      _commandParsingService = commandParsingService;
    }

    public CommandResponse Handle(RegisterKeybindingsCommand command)
    {
      foreach (var keybindingConfig in command.Keybindings)
      {
        // Format command strings defined in keybinding config.
        var formattedCommandStrings = keybindingConfig.CommandList.Select(
          commandString => CommandParsingService.FormatCommand(commandString)
        );

        foreach (var commandString in formattedCommandStrings)
          _commandParsingService.ValidateCommand(commandString);

        // Register all keybindings for a command sequence.
        foreach (var binding in keybindingConfig.BindingList)
          _keybindingService.AddGlobalKeybinding(binding, () =>
          {
            var subjectContainer = _containerService.FocusedContainer;
            var parsedCommands = formattedCommandStrings.Select(
              commandString => _commandParsingService.ParseCommand(commandString, subjectContainer)
            );

            // Invoke commands in sequence on keybinding press. Use `dynamic` to resolve the
            // command type at runtime and allow multiple dispatch.
            foreach (var parsedCommand in parsedCommands)
              _bus.Invoke((dynamic)parsedCommand);
          });
      }

      return CommandResponse.Ok;
    }
  }
}
