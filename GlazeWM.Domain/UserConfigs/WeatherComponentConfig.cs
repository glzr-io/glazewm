namespace GlazeWM.Domain.UserConfigs
{
  public class WeatherComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Latitude to retreive weather.
    /// </summary>
    public float Latitude { get; set; }

    /// <summary>
    /// Longitude to retrieve weather.
    /// </summary>
    public float Longitude { get; set; }

    /// <summary>
    /// How often this component refreshes in milliseconds.
    /// </summary>
    public int RefreshIntervalMs { get; set; } = 60 * 60 * 1000;

    /// <summary>
    /// Default label.
    /// </summary>
    public string Label { get; set; } = "{temperature_celsius}°C";

    /// <summary>
    /// Label to represent sunny weather.
    /// </summary>
    public string LabelSun { get; set; } = "☀️ {temperature_celsius}°C";

    /// <summary>
    /// Label to represent clear weather at night.
    /// </summary>
    public string LabelMoon { get; set; } = "🌙 {temperature_celsius}°C";

    /// <summary>
    /// Label to represent partly cloudy at night.
    /// </summary>
    public string LabelCloudMoon { get; set; } = "🌙☁️ {temperature_celsius}°C";

    /// <summary>
    /// Label to represent partly cloudy.
    /// </summary>
    public string LabelCloudSun { get; set; } = "⛅ {temperature_celsius}°C";

    /// <summary>
    /// Label to represent light rain at night.
    /// </summary>
    public string LabelCloudMoonRain { get; set; } = "🌙🌧️ {temperature_celsius}°C";

    /// <summary>
    /// Label to represent light rain.
    /// </summary>
    public string LabelCloudSunRain { get; set; } = "🌦️ {temperature_celsius}°C";

    /// <summary>
    /// Label to represent heavy rain.
    /// </summary>
    public string LabelCloudRain { get; set; } = "🌧️ {temperature_celsius}°C";

    /// <summary>
    /// Label to represent snow.
    /// </summary>
    public string LabelSnowflake { get; set; } = "❄️ {temperature_celsius}°C";

    /// <summary>
    /// Label to represent a thunderstorm.
    /// </summary>
    public string LabelThunderstorm { get; set; } = "⚡ {temperature_celsius}°C";

    /// <summary>
    /// Label to represent heavy clouds.
    /// </summary>
    public string LabelCloud { get; set; } = "☁️ {temperature_celsius}°C";
  }
}
