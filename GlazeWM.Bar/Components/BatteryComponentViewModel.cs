using System;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Bar.Components
{
  public class BatteryComponentViewModel : ComponentViewModel
  {

    private readonly BatteryComponentConfig _batteryComponentConfig;

    /// <summary>
    /// Format the current power status with the user's formatting config.
    /// </summary>
    public string FormattedPowerStatus => FormatComponent();

    private string FormatComponent()
    {
      WindowsApiService.GetSystemPowerStatus(out WindowsApiService.SYSTEM_POWER_STATUS ps);
      var batteryLevel = ps.BatteryLifePercent.ToString();

      // display the battery level as a 100% if no dedicated battery is available on the device
      if (ps.BatteryFlag == 128)
      {
        return " 100% ";
      }

      if (ps.ACLineStatus == 1)
      {
        return _batteryComponentConfig.Charging.Replace("{battery_level}", batteryLevel);
      }
      else if (ps.SystemStatusFlag == 1)
      {
        return _batteryComponentConfig.PowerSaver.Replace("{battery_level}", batteryLevel);
      }
      else
      {
        return _batteryComponentConfig.Draining.Replace("{battery_level}", batteryLevel);
      }
    }

    public BatteryComponentViewModel(
      BarViewModel parentViewModel,
      BatteryComponentConfig config) : base(parentViewModel, config)
    {
      _batteryComponentConfig = config;

      Observable.Interval(TimeSpan.FromSeconds(3))
        .Subscribe(_ => OnPropertyChanged(nameof(FormattedPowerStatus)));
    }
  }

}
