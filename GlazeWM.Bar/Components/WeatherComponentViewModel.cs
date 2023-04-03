using System;
using System.Globalization;
using System.Reactive.Linq;
using System.Threading.Tasks;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure.Services;

namespace GlazeWM.Bar.Components;

public class WeatherComponentViewModel : ComponentViewModel
{
  private WeatherComponentConfig Config => _componentConfig as WeatherComponentConfig;

  public string FormattedWeather { get; set; }

  public WeatherComponentViewModel(
    BarViewModel parentViewModel,
    WeatherComponentConfig config) : base(parentViewModel, config)
  {
    // This API is updated hourly.
    // ReSharper disable once AsyncVoidLambda
    Observable.Interval(TimeSpan.FromHours(1))
      .TakeUntil(_parentViewModel.WindowClosing)
      .Subscribe(async _ => await UpdateWeatherAsync());

    // Run immediately
    _ = UpdateWeatherAsync();
  }

  private async Task UpdateWeatherAsync()
  {
    var weather = await WeatherService.GetWeatherAsync(Config.Latitude, Config.Longitude, Config.TemperatureUnit);
    var iconText = GetIconText(weather.Icon);
    var temperatureString = weather.Result.CurrentWeather.Temperature.ToString(Config.TemperatureFormat, CultureInfo.InvariantCulture);

    FormattedWeather = string.Format(Config.Format, iconText, temperatureString);
    OnPropertyChanged(nameof(FormattedWeather));
  }

  private string GetIconText(WeatherIcon icon)
  {
    return icon switch
    {
      WeatherIcon.Sun => Config.LabelSun,
      WeatherIcon.Moon => Config.LabelMoon,
      WeatherIcon.CloudSun => Config.LabelCloudSun,
      WeatherIcon.CloudMoon => Config.LabelCloudMoon,
      WeatherIcon.Cloud => Config.LabelCloud,
      WeatherIcon.CloudSunRain => Config.LabelCloudSunRain,
      WeatherIcon.CloudMoonRain => Config.LabelCloudMoonRain,
      WeatherIcon.CloudRain => Config.LabelCloudRain,
      WeatherIcon.Snowflake => Config.LabelSnowflake,
      WeatherIcon.Thunder => Config.LabelThunderstorm,
      _ => ""
    };
  }
}
