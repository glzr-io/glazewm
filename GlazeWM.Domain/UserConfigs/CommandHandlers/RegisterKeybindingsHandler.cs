using System.Linq;
using System.Threading.Tasks;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.UserConfigs.Commands;
using GlazeWM.Domain.Windows;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.UserConfigs.CommandHandlers
{
  internal class RegisterKeybindingsHandler : ICommandHandler<RegisterKeybindingsCommand>
  {
    private readonly Bus _bus;
    private readonly CommandParsingService _commandParsingService;
    private readonly ContainerService _containerService;
    private readonly KeybindingService _keybindingService;
    private readonly WindowService _windowService;

    public RegisterKeybindingsHandler(
      Bus bus,
      CommandParsingService commandParsingService,
      ContainerService containerService,
      KeybindingService keybindingService,
      WindowService windowService)
    {
      _bus = bus;
      _commandParsingService = commandParsingService;
      _containerService = containerService;
      _keybindingService = keybindingService;
      _windowService = windowService;
    }

    public CommandResponse Handle(RegisterKeybindingsCommand command)
    {
      _keybindingService.Reset();

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
            Task.Run(() =>
            {
              // Avoid invoking keybinding if an ignored window currently has focus.
              if (_windowService.IgnoredHandles.Contains(GetForegroundWindow()))
                return;

              var subjectContainer = _containerService.FocusedContainer;
              var subjectContainerId = subjectContainer.Id;

              // Invoke commands in sequence on keybinding press.
              foreach (var commandString in formattedCommandStrings)
              {
                // Avoid calling command if container gets detached. This is to prevent crashes
                // for edge cases like ["close", "move to workspace X"].
                if (subjectContainer.IsDetached())
                  return;

                var parsedCommand = _commandParsingService.ParseCommand(
                  commandString,
                  subjectContainer
                );

                // Use `dynamic` to resolve the command type at runtime and allow multiple dispatch.
                _bus.Invoke((dynamic)parsedCommand);

                // Update subject container in case the reference changes (eg. when going from a tiling to a
                // floating window).
                subjectContainer = _containerService.GetContainerById(subjectContainerId);
              }
            });
          });
      }

      return CommandResponse.Ok;
    }
  }
}
