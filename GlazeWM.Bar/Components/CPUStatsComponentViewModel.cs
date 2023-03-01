using System;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Bar.Components
{
  public class CPUStatsComponentViewModel : ComponentViewModel
  {
    private CPUStatsComponentConfig _config => _componentConfig as CPUStatsComponentConfig;
    private readonly CPUStatsService _cpuStatsService = new();
    public string FormattedText => GetSystemStats();
    private string GetSystemStats()
    {
      return _config.LabelCPU + _cpuStatsService.GetCurrentUtilization().ToString("0.") + "%";
    }
    public CPUStatsComponentViewModel(
      BarViewModel parentViewModel,
      CPUStatsComponentConfig config) : base(parentViewModel, config)
    {
      Observable.Interval(TimeSpan.FromSeconds(2))
        .Subscribe(_ => OnPropertyChanged(nameof(FormattedText)));
    }
  }
}
