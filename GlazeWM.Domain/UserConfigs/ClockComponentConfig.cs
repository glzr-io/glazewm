namespace GlazeWM.Domain.UserConfigs
{
  public class ClockComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Label assigned to the clock component.
    /// </summary>
    public string Label { get; set; } = "{formatted_time}";

    /// <summary>
    /// How to format the current time/date via `DateTime.ToString`.
    /// </summary>
    public string TimeFormatting { get; set; } = "hh:mm tt  ddd MMM d";
  }
}
