using System;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;

namespace GlazeWM.Bar.Components
{
  public class ClockComponentViewModel : ComponentViewModel
  {
    private new readonly ClockComponentConfig _config;

    public string FormattedTime =>
      $"{DateTime.Now.ToShortTimeString()}  {DateTime.Now.ToShortDateString()}";

    public ClockComponentViewModel(BarViewModel parentViewModel, ClockComponentConfig config) : base(parentViewModel, config)
    {
      var updateInterval = TimeSpan.FromSeconds(1);

      Observable.Interval(updateInterval)
        .Subscribe(_ => OnPropertyChanged(nameof(FormattedTime)));
    }
  }
}
