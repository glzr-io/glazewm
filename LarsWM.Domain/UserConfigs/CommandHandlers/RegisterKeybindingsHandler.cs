using System.Linq;
using LarsWM.Domain.UserConfigs.Commands;
using LarsWM.Infrastructure.Bussing;
using LarsWM.Infrastructure.WindowsApi;

namespace LarsWM.Domain.UserConfigs.CommandHandlers
{
  class RegisterKeybindingsHandler : ICommandHandler<RegisterKeybindingsCommand>
  {
    private Bus _bus;
    private KeybindingService _keybindingService;
    private CommandParsingService _commandParsingService;

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
        // Parse command strings defined in keybinding config.
        var parsedCommands = keybindingConfig.CommandList.Select(commandString =>
        {
          try
          {
            commandString = _commandParsingService.FormatCommand(commandString);
            return _commandParsingService.ParseCommand(commandString);
          }
          catch
          {
            throw new FatalUserException($"Invalid command '{commandString}'.");
          }
        });

        // Register all keybindings for a command sequence.
        foreach (var binding in keybindingConfig.BindingList)
          _keybindingService.AddGlobalKeybinding(binding, () =>
          {
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
