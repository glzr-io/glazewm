using System;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Bar.Components
{
  public class RAMStatsComponentViewModel : ComponentViewModel
  {
    private RAMStatsComponentConfig _config => _componentConfig as RAMStatsComponentConfig;
    private readonly RAMStatsService _ramStatsService = new();
    public string FormattedText => GetSystemStats();
    private string GetSystemStats()
    {
      return _config.LabelRAM + _ramStatsService.GetCurrentUtilization().ToString("0.") + "%";
    }
    public RAMStatsComponentViewModel(
      BarViewModel parentViewModel,
      RAMStatsComponentConfig config) : base(parentViewModel, config)
    {
      Observable.Interval(TimeSpan.FromSeconds(2))
        .Subscribe(_ => OnPropertyChanged(nameof(FormattedText)));
    }
  }
}
