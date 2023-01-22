using System.Collections.Generic;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.UserConfigs.Commands
{
  // TODO: Consider renaming to `RegisterUserKeybindingsCommand`.
  // TODO: Perhaps call command for each keybinding config.
  public class RegisterKeybindingsCommand : Command
  {
    public List<KeybindingConfig> Keybindings { get; }

    public RegisterKeybindingsCommand(List<KeybindingConfig> keybindings)
    {
      Keybindings = keybindings;
    }
  }
}
