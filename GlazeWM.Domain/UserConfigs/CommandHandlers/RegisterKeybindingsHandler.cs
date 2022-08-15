using System.Linq;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.UserConfigs.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.UserConfigs.CommandHandlers
{
  internal class RegisterKeybindingsHandler : ICommandHandler<RegisterKeybindingsCommand>
  {
    private readonly Bus _bus;
    private readonly CommandParsingService _commandParsingService;
    private readonly ContainerService _containerService;
    private readonly KeybindingService _keybindingService;

    public RegisterKeybindingsHandler(
      Bus bus,
      CommandParsingService commandParsingService,
      ContainerService containerService,
      KeybindingService keybindingService
    )
    {
      _bus = bus;
      _commandParsingService = commandParsingService;
      _containerService = containerService;
      _keybindingService = keybindingService;
    }

    public CommandResponse Handle(RegisterKeybindingsCommand command)
    {
      foreach (var keybindingConfig in command.Keybindings)
      {
        // Format command strings defined in keybinding config.
        var formattedCommandStrings = keybindingConfig.CommandList.Select(
          commandString => CommandParsingService.FormatCommand(commandString)
        );

        // Register all keybindings for a command sequence.
        foreach (var binding in keybindingConfig.BindingList)
          _keybindingService.AddGlobalKeybinding(binding, () =>
          {
            // Avoid invoking keybinding if focus is not synced (eg. if an unmanaged window has
            // focus).
            if (!_containerService.IsFocusSynced)
              return;

            // Invoke commands in sequence on keybinding press.
            foreach (var commandString in formattedCommandStrings)
            {
              var subjectContainer = _containerService.FocusedContainer;

              // Use `dynamic` to resolve the command type at runtime and allow multiple dispatch.
              var parsedCommand = _commandParsingService.ParseCommand(commandString, subjectContainer);
              _bus.Invoke((dynamic)parsedCommand);
            }
          });
      }

      return CommandResponse.Ok;
    }
  }
}
