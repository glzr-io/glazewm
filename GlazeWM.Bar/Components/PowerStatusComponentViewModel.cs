using System;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Bar.Components
{
  public class PowerStatusComponentViewModel : ComponentViewModel
  {

    private readonly PowerStatusComponentConfig _powerStatusConfig;

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
        return _powerStatusConfig.Charging.Replace("{battery_level}", batteryLevel);
      }
      else if (int.Parse(batteryLevel) <= 20)
      {
        return _powerStatusConfig.Low.Replace("{battery_level}", batteryLevel);
      }
      else
      {
        return _powerStatusConfig.Draining.Replace("{battery_level}", batteryLevel);
      }
    }

    public PowerStatusComponentViewModel(
      BarViewModel parentViewModel,
      PowerStatusComponentConfig config) : base(parentViewModel, config)
    {
      _powerStatusConfig = config;


      Observable.Interval(TimeSpan.FromSeconds(1))
        .Subscribe(_ => OnPropertyChanged(nameof(FormattedPowerStatus)));
    }
  }

}
