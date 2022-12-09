using GlazeWM.Domain.Common.Commands;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.UserConfigs.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Common.CommandHandlers
{
  public class SetBindingModeHandler : ICommandHandler<SetBindingModeCommand>
  {
    private readonly Bus _bus;
    private readonly UserConfigService _userConfigService;

    public SetBindingModeHandler(Bus bus, UserConfigService userConfigService)
    {
      _bus = bus;
      _userConfigService = userConfigService;
    }

    public CommandResponse Handle(SetBindingModeCommand command)
    {
      var bindingModeName = command.BindingModeName;

      // If binding mode is "none", then reset keybindings to default.
      if (bindingModeName == "none")
      {
        var defaultKeybindings = _userConfigService.Keybindings;
        _bus.Invoke(new RegisterKeybindingsCommand(defaultKeybindings));
        return CommandResponse.Ok;
      }

      // Otherwise, set keybindings to those defined by the binding mode.
      var bindingMode = _userConfigService.GetBindingModeByName(bindingModeName);
      _bus.Invoke(new RegisterKeybindingsCommand(bindingMode.Keybindings));

      return CommandResponse.Ok;
    }
  }
}
