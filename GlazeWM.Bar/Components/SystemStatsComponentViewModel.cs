using System;
using System.Reactive.Linq;

namespace GlazeWM.Bar.Components
{
  public class SystemStatsComponentViewModel : ComponentViewModel
  {
    private SystemStatsComponentConfig _config => _componentConfig as SystemStatsComponentConfig;
    public string FormattedText => GetWeather();

    private string GetWeather()
    {
      return "ABC";
    }

    public SystemStatsComponentViewModel(
      BarViewModel parentViewModel,
      SystemStatsComponentConfig config) : base(parentViewModel, config)
    {
      Observable.Interval(TimeSpan.FromSeconds(3))
        .Subscribe(_ => OnPropertyChanged(nameof(FormattedText)));
    }
  }
}
