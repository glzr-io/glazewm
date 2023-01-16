using System;
using System.Globalization;

namespace GlazeWM.Domain.UserConfigs
{
  public class ClockComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// How to format the current time/date via `DateTime.ToString`.
    /// </summary>
    public string TimeFormatting { get; set; } = "hh:mm tt  ddd MMM d";
  }
}
