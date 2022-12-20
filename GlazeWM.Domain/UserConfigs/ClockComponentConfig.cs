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
    public bool WeekOfYear { get; set; } = false;
    /// <summary>
    /// See CalendarWeekRule
    /// FirstDay = 0    
    /// FirstFullWeek = 1
    /// FirstFourDayWeek = 2
    /// </summary>
    public CalendarWeekRule CalendarWeekRule { get; set; } = System.Globalization.CalendarWeekRule.FirstFullWeek;

    /// <summary>
    /// First day of week: 0..6 => Sunday..Saturday
    /// </summary>
    public DayOfWeek FirstDayOfWeek { get; set; } = DayOfWeek.Monday;
  }
}
