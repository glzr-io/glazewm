using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;

namespace GlazeWM.Bar.Components
{
  public class SystemStatsComponentViewModel : ComponentViewModel
  {
    private SystemStatsComponentConfig _config => _componentConfig as SystemStatsComponentConfig;
    public string FormattedText => GetSystemStats();
    private string GetSystemStats()
    {
      var x = cpuCounter.NextValue();
      double total = new Microsoft.VisualBasic.Devices.ComputerInfo().TotalPhysicalMemory;
      var used = 1024.0 * 1024.0 * ramCounter.NextValue();
      var ramUsage = 100.0 * (total - used) / total;
      var gpuCounters = GetGPUCounters();
      var gpuUsage = GetGPUUsage(gpuCounters);
      return _config.LabelCPU + x.ToString("0.") + "%  " + _config.LabelRAM + ramUsage.ToString("0.") + "%  " + _config.LabelGPU + gpuUsage.ToString("0.") + "%";
    }
    private PerformanceCounter cpuCounter = new PerformanceCounter("Processor Information", "% Processor Utility", "_Total");
    private PerformanceCounter ramCounter = new PerformanceCounter("Memory", "Available MBytes");

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

    public SystemStatsComponentViewModel(
      BarViewModel parentViewModel,
      SystemStatsComponentConfig config) : base(parentViewModel, config)
    {
      Observable.Interval(TimeSpan.FromSeconds(2))
        .Subscribe(_ => OnPropertyChanged(nameof(FormattedText)));
    }
  }
}
