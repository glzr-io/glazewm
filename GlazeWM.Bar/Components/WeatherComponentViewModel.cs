using System;
using System.Collections.Generic;
using System.Globalization;
using System.Reactive.Linq;
using System.Threading.Tasks;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure.Services;

namespace GlazeWM.Bar.Components
{
  public class WeatherComponentViewModel : ComponentViewModel
  {
    private readonly WeatherComponentConfig _config;

    private LabelViewModel _label;
    public LabelViewModel Label
    {
      get => _label;
      protected set => SetField(ref _label, value);
    }

    public WeatherComponentViewModel(
      BarViewModel parentViewModel,
      WeatherComponentConfig config) : base(parentViewModel, config)
    {
      _config = config;

      Observable.Timer(
        TimeSpan.Zero,
        TimeSpan.FromMilliseconds(_config.RefreshIntervalMs)
      )
        .TakeUntil(_parentViewModel.WindowClosing)
        .Subscribe(async (_) => await UpdateLabel());
    }

    private async Task UpdateLabel()
    {
      var weather = await WeatherService.GetWeatherAsync(_config.Latitude, _config.Longitude);

      var variableDictionary = new Dictionary<string, Func<string>>()
      {
        {
          "temperature_celsius",
          () => weather.Result.CurrentWeather.Temperature.ToString("0", CultureInfo.InvariantCulture)
        },
        {
          "temperature_fahrenheit",
          () =>
            WeatherService
              .ToFahrenheit(weather.Result.CurrentWeather.Temperature)
              .ToString("0", CultureInfo.InvariantCulture)
        }
      };

      var labelString = GetLabelStringFromStatus(weather.Status);
      Label = XamlHelper.ParseLabel(labelString, variableDictionary, this);
    }

    private string GetLabelStringFromStatus(WeatherStatus status)
    {
      return status switch
      {
        WeatherStatus.Sun => _config.LabelSun,
        WeatherStatus.Moon => _config.LabelMoon,
        WeatherStatus.CloudSun => _config.LabelCloudSun,
        WeatherStatus.CloudMoon => _config.LabelCloudMoon,
        WeatherStatus.Cloud => _config.LabelCloud,
        WeatherStatus.CloudSunRain => _config.LabelCloudSunRain,
        WeatherStatus.CloudMoonRain => _config.LabelCloudMoonRain,
        WeatherStatus.CloudRain => _config.LabelCloudRain,
        WeatherStatus.Snowflake => _config.LabelSnowflake,
        WeatherStatus.Thunder => _config.LabelThunderstorm,
        _ => _config.Label
      };
    }
  }
}
