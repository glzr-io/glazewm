using System;
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
  }

  public class MeteoCurrentWeatherResult
  {
    public float temperature { get; set; }
    public int weathercode { get; set; }
  }

  public class WeatherComponentViewModel : ComponentViewModel
  {
    private WeatherComponentConfig _config => _componentConfig as WeatherComponentConfig;
    public string FormattedWeather => GetWeather();

    private string GetWeather()
    {
      HttpClient client = new HttpClient();
      var lat = _config.Latitude.ToString();
      var lng = _config.Longitude.ToString();
      var res = client.GetStringAsync("https://api.open-meteo.com/v1/forecast?latitude=" + lat + "&longitude=" + lng + "&temperature_unit=" + _config.TemperatureUnit + "&current_weather=true").Result;
      var weather = JsonSerializer.Deserialize<MeteoWeatherResult>(res);
      return weather.current_weather.temperature.ToString();
    }

    public WeatherComponentViewModel(
      BarViewModel parentViewModel,
      WeatherComponentConfig config) : base(parentViewModel, config)
    {
      Observable.Interval(TimeSpan.FromSeconds(3))
        .Subscribe(_ => OnPropertyChanged(nameof(FormattedWeather)));
    }
  }
}
