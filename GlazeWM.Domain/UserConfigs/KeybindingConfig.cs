using System.Collections.Generic;

namespace GlazeWM.Domain.UserConfigs
{
  public class KeybindingConfig
  {
    public string Binding { get; set; }

    public List<string> Bindings { get; set; }

    public List<string> BindingList =>
      Binding != null ? new List<string> { Binding } : Bindings;

    public string Command { get; set; }

    public List<string> Commands { get; set; }

    public List<string> CommandList =>
      Command != null ? new List<string> { Command } : Commands;
  }
}
