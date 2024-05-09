using System.Security.Policy;

namespace GlazeWM.Domain.UserConfigs
{
  public class BindingModeComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Label assigned to the binding mode component.
    /// </summary>
    public string Label { get; set; } = "{binding_mode}";

    public string DefaultLabel { get; set; } = "";
  }
}
