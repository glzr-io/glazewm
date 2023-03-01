using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;

namespace GlazeWM.Bar.Components
{
  public class GPUStatsComponentViewModel : ComponentViewModel
  {
    private GPUStatsComponentConfig _config => _componentConfig as GPUStatsComponentConfig;
    public string FormattedText => GetSystemStats();
    private string GetSystemStats()
    {
      var gpuCounters = GetGPUCounters();
      var gpuUsage = GetGPUUsage(gpuCounters);
      return _config.LabelGPU + gpuUsage.ToString("0.") + "%";
    }
    public static List<PerformanceCounter> GetGPUCounters()
    {
      var category = new PerformanceCounterCategory("GPU Engine");
      var counterNames = category.GetInstanceNames();

      return counterNames
        .Where(counterName => counterName.EndsWith("engtype_3D"))
        .SelectMany(counterName => category.GetCounters(counterName))
        .Where(counter => counter.CounterName.Equals("Utilization Percentage"))
        .ToList();
    }

    public static float GetGPUUsage(List<PerformanceCounter> gpuCounters)
    {
      gpuCounters.ForEach(x => x.NextValue());
      return (float)gpuCounters.Sum(x => x.NextValue());
    }

    public GPUStatsComponentViewModel(
      BarViewModel parentViewModel,
      GPUStatsComponentConfig config) : base(parentViewModel, config)
    {
      Observable.Interval(TimeSpan.FromSeconds(2))
        .Subscribe(_ => OnPropertyChanged(nameof(FormattedText)));
    }
  }
}
