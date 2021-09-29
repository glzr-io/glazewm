using System;
using System.Reactive.Linq;

namespace GlazeWM.Bar.Components
{
  public class ClockComponentViewModel : ComponentViewModel
  {
    public string FormattedTime =>
      $"{DateTime.Now.ToShortTimeString()} {DateTime.Now.ToShortDateString()}";

    public ClockComponentViewModel(BarViewModel parentViewModel) : base(parentViewModel)
    {
      var updateInterval = TimeSpan.FromSeconds(1);

      Observable.Interval(updateInterval)
        .Subscribe(_ => OnPropertyChanged(nameof(FormattedTime)));
    }
  }
}
