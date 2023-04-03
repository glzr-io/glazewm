using GlazeWM.Infrastructure.Services;

namespace GlazeWM.Domain.UserConfigs;

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
  /// Format of the final string.
  /// </summary>
  public string Format { get; set; } = "{0}{1}°C";

  /// <summary>
  /// Unit of measurement. Either "celsius" or "fahrenheit"
  /// </summary>
  public TemperatureUnit TemperatureUnit { get; set; } = TemperatureUnit.Celsius;

  /// <summary>
  /// Format string used for the temperature.
  /// </summary>
  public string TemperatureFormat { get; set; } = "0";

  /// <summary>
  /// Icon to represent sunny weather.
  /// </summary>
  public string LabelSun { get; set; } = "☀️";

  /// <summary>
  /// Icon to represent clear weather at night.
  /// </summary>
  public string LabelMoon { get; set; } = "🌙";

  /// <summary>
  /// Icon to represent partly cloudy at night.
  /// </summary>
  public string LabelCloudMoon { get; set; } = "🌙☁️";

  /// <summary>
  /// Icon to represent partly cloudy.
  /// </summary>
  public string LabelCloudSun { get; set; } = "⛅";

  /// <summary>
  /// Icon to represent light rain at night.
  /// </summary>
  public string LabelCloudMoonRain { get; set; } = "🌙🌧️";

  /// <summary>
  /// Icon to represent light rain.
  /// </summary>
  public string LabelCloudSunRain { get; set; } = "🌦️";

  /// <summary>
  /// Icon to represent heavy rain.
  /// </summary>
  public string LabelCloudRain { get; set; } = "🌧️";

  /// <summary>
  /// Icon to represent snow.
  /// </summary>
  public string LabelSnowflake { get; set; } = "❄️";

  /// <summary>
  /// Icon to represent a thunderstorm.
  /// </summary>
  public string LabelThunderstorm { get; set; } = "⚡";

  /// <summary>
  /// Icon to represent heavy clouds.
  /// </summary>
  public string LabelCloud { get; set; } = "☁️";
}
