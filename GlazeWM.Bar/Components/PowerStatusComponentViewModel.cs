using System;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Bar.Components
{
  public class PowerStatusComponentViewModel : ComponentViewModel
  {

    public string PowerStatus => RetrieveCurrentPowerStatus();

    private string RetrieveCurrentPowerStatus()
    {
      WindowsApiService.GetSystemPowerStatus(out WindowsApiService.SYSTEM_POWER_STATUS status);

      return $" {status.BatteryLifePercent}% ";
    }

    public PowerStatusComponentViewModel(
      BarViewModel parentViewModel,
      PowerStatusComponentConfig config) : base(parentViewModel, config)
    {
      Observable.Interval(TimeSpan.FromSeconds(1))
        .Subscribe(_ => OnPropertyChanged(nameof(PowerStatus)));
    }
  }

}
