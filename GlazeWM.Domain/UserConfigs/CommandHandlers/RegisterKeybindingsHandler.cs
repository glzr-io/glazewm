using System.Linq;
using GlazeWM.Domain.UserConfigs.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.UserConfigs.CommandHandlers
{
  class RegisterKeybindingsHandler : ICommandHandler<RegisterKeybindingsCommand>
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
        // Parse command strings defined in keybinding config. Calling `ToList()` is necessary here,
        // otherwise parsing errors get delayed until keybinding is invoked instead of at startup.
        var parsedCommands = keybindingConfig.CommandList.ConvertAll(commandString =>
        {
          commandString = CommandParsingService.FormatCommand(commandString);
          return _commandParsingService.ParseCommand(commandString);
        })
;

        // Register all keybindings for a command sequence.
        foreach (var binding in keybindingConfig.BindingList)
          _keybindingService.AddGlobalKeybinding(binding, () =>
          {
            // Invoke commands in sequence on keybinding press. Use `dynamic` to resolve the
            // command type at runtime and allow multiple dispatch.
            foreach (var parsedCommand in parsedCommands)
              Bus.Invoke((dynamic)parsedCommand);
          });
      }

      return CommandResponse.Ok;
    }
  }
}
