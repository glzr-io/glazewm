namespace GlazeWM.Domain.UserConfigs
{
  public class WeatherComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Text to display.
    /// </summary>
    public string Text { get; set; } = "Hello world!";
    /// <summary>
    /// Latitude to retreive weather.
    /// </summary>
    public float Latitude { get; set; } = 40.7128f;
    /// <summary>
    /// Longitude to retrieve weather.
    /// </summary>
    public float Longitude { get; set; } = 74.0060f;
    /// <summary>
    /// Longitude to retrieve weather.
    /// </summary>
    public string TemperatureUnit { get; set; } = "celcius";
  }
}
