using System;
using System.Collections.Generic;
using System.Globalization;
using System.Net.Http;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Threading.Tasks;
using static GlazeWM.Infrastructure.Services.WeatherStatus;

namespace GlazeWM.Infrastructure.Services
{
  /// <summary>
  /// Service that queries current weather status using the OpenMeteo API.
  /// </summary>
  public static class WeatherService
  {
    /// <summary>
    /// Retrieves the current weather status asynchronously.
    /// </summary>
    /// <param name="latitude">Current latitude.</param>
    /// <param name="longitude">Current longitude.</param>
    public static async Task<WeatherServiceResult> GetWeatherAsync(float latitude, float longitude)
    {
      var lat = latitude.ToString(CultureInfo.InvariantCulture);
      var lng = longitude.ToString(CultureInfo.InvariantCulture);

      using var client = new HttpClient();

      var res = await client.GetStringAsync(
        "https://api.open-meteo.com/v1/forecast?latitude=" +
          lat +
          "&longitude=" +
          lng +
          "&temperature_unit=celsius&current_weather=true&daily=sunset,sunrise&timezone=auto"
      );

      var weather = JsonSerializer.Deserialize<MeteoWeatherResult>(res);
      var isDaylight = IsDaytime(weather.Daily);
      var status = GetWeatherStatus(weather.CurrentWeather.WeatherCode, isDaylight);

      return new WeatherServiceResult(status, weather);
    }

    private static bool IsDaytime(MeteoDailyResult times)
    {
      var now = DateTime.Now;
      var sunsetToday = times.Sunset.Find(t => t.Day == now.Day);
      var sunriseToday = times.Sunrise.Find(t => t.Day == now.Day);
      return now > sunriseToday && now < sunsetToday;
    }

    public static double ToFahrenheit(double celsius)
    {
      return (celsius * 1.8) + 32;
    }

    private static WeatherStatus GetWeatherStatus(int code, bool isDaytime)
    {
      return code switch
      {
        // https://open-meteo.com/en/docs
        0 => isDaytime ? Sun : Moon,
        < 3 => isDaytime ? CloudSun : CloudMoon,
        < 50 => Cloud,
        < 60 => isDaytime ? CloudSunRain : CloudMoonRain,
        < 70 => CloudRain,
        < 80 => Snowflake,
        < 83 => CloudRain,
        < 87 => Snowflake,
        < 99 => Thunder,
        _ => Sun
      };
    }
  }

  /// <summary>
  /// Result of individual weather service query.
  /// </summary>
  /// <param name="Status">Weather status.</param>
  /// <param name="Result">Result of the weather lookup.</param>
  public record WeatherServiceResult(WeatherStatus Status, MeteoWeatherResult Result);

  /// <summary>
  /// Describes the icon to use for a given returned weather code.
  /// </summary>
  public enum WeatherStatus
  {
    Sun,
    Moon,
    CloudSun,
    CloudMoon,
    Cloud,
    CloudSunRain,
    CloudMoonRain,
    CloudRain,
    Snowflake,
    Thunder
  }

  /// <summary>
  /// Temperature Unit returned from the API.
  /// </summary>
  public enum TemperatureUnit
  {
    /// <summary>
    /// Temperature is returned in Celsius (C).
    /// </summary>
    Celsius,

    /// <summary>
    /// Temperature is returned in Fahrenheit (F).
    /// </summary>
    Fahrenheit
  }

  public record MeteoWeatherResult
  {
    [JsonPropertyName("latitude")]
    public float Latitude { get; set; }

    [JsonPropertyName("longitude")]
    public float Longitude { get; set; }

    [JsonPropertyName("current_weather")]
    public MeteoCurrentWeatherResult CurrentWeather { get; set; }

    [JsonPropertyName("daily")]
    public MeteoDailyResult Daily { get; set; }
  }

  public record MeteoDailyResult
  {
    [JsonPropertyName("sunset")]
    public List<DateTime> Sunset { get; set; }

    [JsonPropertyName("sunrise")]
    public List<DateTime> Sunrise { get; set; }
  }

  public record MeteoCurrentWeatherResult
  {
    [JsonPropertyName("temperature")]
    public float Temperature { get; set; }

    [JsonPropertyName("weathercode")]
    public int WeatherCode { get; set; }

    [JsonPropertyName("time")]
    public DateTime Time { get; set; }
  }
}
