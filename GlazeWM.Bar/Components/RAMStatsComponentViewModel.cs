using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;

namespace GlazeWM.Bar.Components
{
  public class RAMStatsComponentViewModel : ComponentViewModel
  {
    private RAMStatsComponentConfig _config => _componentConfig as RAMStatsComponentConfig;
    public string FormattedText => GetSystemStats();
    private string GetSystemStats()
    {
      double total = new Microsoft.VisualBasic.Devices.ComputerInfo().TotalPhysicalMemory;
      var used = 1024.0 * 1024.0 * ramCounter.NextValue();
      var ramUsage = 100.0 * (total - used) / total;
      return _config.LabelRAM + ramUsage.ToString("0.") + "%";
    }
    private PerformanceCounter ramCounter = new PerformanceCounter("Memory", "Available MBytes");

    public RAMStatsComponentViewModel(
      BarViewModel parentViewModel,
      RAMStatsComponentConfig config) : base(parentViewModel, config)
    {
      Observable.Interval(TimeSpan.FromSeconds(2))
        .Subscribe(_ => OnPropertyChanged(nameof(FormattedText)));
    }
  }
}
