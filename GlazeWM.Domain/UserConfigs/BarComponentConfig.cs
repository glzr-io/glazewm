using System;
using System.ComponentModel.DataAnnotations;

namespace GlazeWM.Domain.UserConfigs
{
  public class BarComponentConfig : CommonBarAttributes
  {
    [Required]
    public string Type { get; set; }

    public string Margin { get; set; } = "0 10 0 0";

    public string TimeFormatting { get; set; }

    /// <summary>
    /// The YAML parsing library doesn't have proper support for polymorphic objects. Therefore,
    /// a base class with all properties of derived classes is needed, which is then upcasted into
    /// the correct type.
    /// </summary>
    public BarComponentConfig Upcast()
    {
      switch (Type)
      {
        case "workspaces":
          return new WorkspacesComponentConfig();

        case "clock":
          return new ClockComponentConfig
          {
            TimeFormatting = TimeFormatting,
          };

        default:
          throw new ArgumentException();
      }
    }
  }
}
