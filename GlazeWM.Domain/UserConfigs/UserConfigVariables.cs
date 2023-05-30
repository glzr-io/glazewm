using System.Collections.Generic;

namespace GlazeWM.Domain.UserConfigs
{
  public class UserConfigVariables
  {
    public List<GlobalVariablesConfig> Globals { get; set; } = new();
  }
}
