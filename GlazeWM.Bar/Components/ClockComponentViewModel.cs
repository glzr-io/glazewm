using System;
using System.Globalization;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;

namespace GlazeWM.Bar.Components
{
  public class ClockComponentViewModel : ComponentViewModel
  {
    private ClockComponentConfig _config => _componentConfig as ClockComponentConfig;
    private string _timeFormatting => _config.TimeFormatting;

    /// <summary>
    /// Format the current time with the user's formatting config.
    /// </summary>
    public string FormattedTime
    {
      get
      {
        var now = DateTime.Now;
        var dateString = now.ToString(_timeFormatting, CultureInfo.InvariantCulture);
        var additionalInfo = "";
        if (_config.WeekOfYear)
        {
          additionalInfo = additionalInfo + CultureInfo.CurrentCulture.Calendar.GetWeekOfYear(now, _config.CalendarWeekRule, _config.FirstDayOfWeek) + " CW";
        }
        return (dateString + " (" + additionalInfo + ")");
      }
    }

    public ClockComponentViewModel(
      BarViewModel parentViewModel,
      ClockComponentConfig config) : base(parentViewModel, config)
    {
      // Update the displayed time every second.
      var updateInterval = TimeSpan.FromSeconds(1);

      Observable.Interval(updateInterval)
        .Subscribe(_ => OnPropertyChanged(nameof(FormattedTime)));
    }
  }
}
