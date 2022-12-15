using System;
using System.Globalization;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;

namespace GlazeWM.Bar.Components
{
  public class BindingModeComponentViewModel : ComponentViewModel
  {
    public BindingModeComponentViewModel(
      BarViewModel parentViewModel,
      BindingModeComponentConfig config) : base(parentViewModel, config)
    {
      // Update the displayed time every second.
      var updateInterval = TimeSpan.FromSeconds(1);

      // Observable.Interval(updateInterval)
      //   .Subscribe(_ => OnPropertyChanged(nameof("j")));
    }
  }
}
