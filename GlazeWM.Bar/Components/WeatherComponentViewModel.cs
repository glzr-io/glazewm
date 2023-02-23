using System;
using System.Collections.Generic;
using System.IO;
using System.Net;
using System.Net.Http;
using System.Reactive.Linq;
using System.Text.Json;
using GlazeWM.Domain.UserConfigs;

namespace GlazeWM.Bar.Components
{
  public class MeteoWeatherResult
  {
    public float latitude { get; set; }
    public float longitude { get; set; }
    public MeteoCurrentWeatherResult current_weather { get; set; }
    public MeteoDailyResult daily { get; set; }
  }

  public class MeteoDailyResult
  {
    public List<DateTime> sunset { get; set; }
    public List<DateTime> sunrise { get; set; }
  }

  public class MeteoCurrentWeatherResult
  {
    public float temperature { get; set; }
    public int weathercode { get; set; }
    public DateTime time { get; set; }
  }

  public class WeatherComponentViewModel : ComponentViewModel
  {
    private WeatherComponentConfig _config => _componentConfig as WeatherComponentConfig;
    public string FormattedWeather => GetWeather();

    private string GetWeather()
    {
      var lat = _config.Latitude.ToString();
      var lng = _config.Longitude.ToString();

      HttpClient client = new HttpClient();
      var res = client.GetStringAsync("https://api.open-meteo.com/v1/forecast?latitude=" + lat + "&longitude=" + lng + "&temperature_unit=" + _config.TemperatureUnit + "&current_weather=true&daily=sunset,sunrise&timezone=Europe/Berlin").Result;
      var weather = JsonSerializer.Deserialize<MeteoWeatherResult>(res);

      var isDaylight = isDaytime(weather.daily);
      var icon = GetWeatherIcon(weather.current_weather.weathercode, isDaylight);
      return icon + weather.current_weather.temperature.ToString("0") + "°";
    }

    private bool isDaytime(MeteoDailyResult times)
    {
      var now = DateTime.Now;
      var sunsetToday = times.sunset.Find(t => t.Day == now.Day);
      var sunriseToday = times.sunrise.Find(t => t.Day == now.Day);
      return now > sunriseToday && now < sunsetToday;
    }

    private string GetWeatherIcon(int code, bool isDaytime)
    {
      // https://open-meteo.com/en/docs
      if (code == 0)
        return isDaytime ? _config.LabelSun : _config.LabelMoon;
      else if (code < 3)
        return isDaytime ? _config.LabelCloudSun : _config.LabelCloudMoon;
      else if (code < 50)
        return _config.LabelCloud;
      else if (code < 60)
        return isDaytime ? ":CloudSunRain" : ":CloudMoonRain:";
      else if (code < 70)
        return "CloudRain:";
      else if (code < 80)
        return ":Snowflake:";
      else if (code < 83)
        return "CloudRain:";
      else if (code < 87)
        return ":Snowflake:";
      else if (code < 99)
        return ":Thunderstorm:";
      else
        return "";
    }

    public WeatherComponentViewModel(
      BarViewModel parentViewModel,
      WeatherComponentConfig config) : base(parentViewModel, config)
    {
      Observable.Interval(TimeSpan.FromSeconds(3 * 60))
        .Subscribe(_ => OnPropertyChanged(nameof(FormattedWeather)));
    }
  }
}
