using GlazeWM.Domain.Common.Commands;
using GlazeWM.Domain.Common.Events;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.UserConfigs.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Common.CommandHandlers
{
  public class SetBindingModeHandler : ICommandHandler<SetBindingModeCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;
    private readonly UserConfigService _userConfigService;

    public SetBindingModeHandler(
      Bus bus,
      ContainerService containerService,
      UserConfigService userConfigService)
    {
      _bus = bus;
      _containerService = containerService;
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

        _containerService.ActiveBindingMode = null;
        _bus.Emit(new BindingModeChangedEvent(null));

        return CommandResponse.Ok;
      }

      // Otherwise, set keybindings to those defined by the binding mode.
      var bindingMode = _userConfigService.GetBindingModeByName(bindingModeName);
      _bus.Invoke(new RegisterKeybindingsCommand(bindingMode.Keybindings));

      _containerService.ActiveBindingMode = bindingModeName;
      _bus.Emit(new BindingModeChangedEvent(bindingModeName));

      return CommandResponse.Ok;
    }
  }
}
