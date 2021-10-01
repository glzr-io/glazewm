using System;
using System.Globalization;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;

namespace GlazeWM.Bar.Components
{
  public class ClockComponentViewModel : ComponentViewModel
  {
    private ClockComponentConfig _config => _baseConfig as ClockComponentConfig;
    private string _timeFormatting => _config.TimeFormatting;

    /// <summary>
    /// Format the current time with the user's formatting config.
    /// </summary>
    public string FormattedTime => DateTime.Now.ToString(_timeFormatting, CultureInfo.InvariantCulture);

    public ClockComponentViewModel(BarViewModel parentViewModel, ClockComponentConfig config) : base(parentViewModel, config)
    {
      // Update the displayed time every second.
      var updateInterval = TimeSpan.FromSeconds(1);

      Observable.Interval(updateInterval)
        .Subscribe(_ => OnPropertyChanged(nameof(FormattedTime)));
    }
  }
}
