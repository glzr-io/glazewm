using System;
using System.Linq;
using System.Threading.Tasks;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.UserConfigs.Commands;
using GlazeWM.Domain.Windows;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Commands;
using GlazeWM.Infrastructure.WindowsApi;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.UserConfigs.CommandHandlers
{
  internal sealed class RegisterKeybindingsHandler : ICommandHandler<RegisterKeybindingsCommand>
  {
    private readonly Bus _bus;
    private readonly KeybindingService _keybindingService;
    private readonly WindowService _windowService;

    public RegisterKeybindingsHandler(
      Bus bus,
      KeybindingService keybindingService,
      WindowService windowService)
    {
      _bus = bus;
      _keybindingService = keybindingService;
      _windowService = windowService;
    }

    public CommandResponse Handle(RegisterKeybindingsCommand command)
    {
      _keybindingService.Reset();

      foreach (var keybindingConfig in command.Keybindings)
      {
        // Format command strings defined in keybinding config.
        var commandStrings = keybindingConfig.CommandList.Select(
          CommandParsingService.FormatCommand
        );

        // Register all keybindings for a command sequence.
        foreach (var binding in keybindingConfig.BindingList)
          _keybindingService.AddGlobalKeybinding(binding, () =>
          {
            Task.Run(() =>
            {
              lock (_bus.LockObj)
              {
                try
                {
                  // Avoid invoking keybinding if an ignored window currently has focus.
                  if (_windowService.IgnoredHandles.Contains(GetForegroundWindow()))
                    return;

                  _bus.Invoke(new RunWithSubjectContainerCommand(commandStrings));
                  _bus.Invoke(new RedrawContainersCommand());
                }
                catch (Exception e)
                {
                  _bus.Invoke(new HandleFatalExceptionCommand(e));
                  throw;
                }
              }
            });
          });
      }

      return CommandResponse.Ok;
    }
  }
}
