using System;
using System.Diagnostics;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;

namespace GlazeWM.Bar.Components
{
  public class CPUStatsComponentViewModel : ComponentViewModel
  {
    private CPUStatsComponentConfig _config => _componentConfig as CPUStatsComponentConfig;
    public string FormattedText => GetSystemStats();
    private string GetSystemStats()
    {
      var x = cpuCounter.NextValue();
      return _config.LabelCPU + x.ToString("0.") + "%";
    }
    private PerformanceCounter cpuCounter = new PerformanceCounter("Processor Information", "% Processor Utility", "_Total");

    public CPUStatsComponentViewModel(
      BarViewModel parentViewModel,
      CPUStatsComponentConfig config) : base(parentViewModel, config)
    {
      Observable.Interval(TimeSpan.FromSeconds(2))
        .Subscribe(_ => OnPropertyChanged(nameof(FormattedText)));
    }
  }
}
