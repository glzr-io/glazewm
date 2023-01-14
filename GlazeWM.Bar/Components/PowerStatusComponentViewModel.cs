using System;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Bar.Components
{
  public class PowerStatusComponentViewModel : ComponentViewModel
  {

    private PowerStatusComponentConfig _powerStatusConfig;

    public string FormattedPowerStatus => FormatComponent();

    private WindowsApiService.SYSTEM_POWER_STATUS RetrievePowerStatus()
    {
        WindowsApiService.GetSystemPowerStatus(out WindowsApiService.SYSTEM_POWER_STATUS status);
        return status;
    }

    private string FormatComponent()
    {
      var ps = RetrievePowerStatus();
      var batteryLevel = ps.BatteryLifePercent.ToString();

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
