using System.Collections.Generic;

namespace GlazeWM.Domain.UserConfigs
{
  public class BindingMode
  {
    public string Name { get; set; }

    public List<KeybindingConfig> Keybindings { get; set; } = new();
  }
}
