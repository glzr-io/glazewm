using System;
using System.Globalization;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Bar.Components;

public class CpuPercentComponentViewModel : ComponentViewModel
{
  private CpuPercentComponentConfig Config => _componentConfig as CpuPercentComponentConfig;
  private readonly CpuStatsService _cpuStatsService;

  public string FormattedText => GetFormattedText();

  public CpuPercentComponentViewModel(BarViewModel parentViewModel, CpuPercentComponentConfig config) : base(parentViewModel, config)
  {
    _cpuStatsService = ServiceLocator.GetRequiredService<CpuStatsService>();

    // Update the displayed time every second.
    var updateInterval = TimeSpan.FromSeconds(1);

    Observable
      .Interval(updateInterval)
      .TakeUntil(_parentViewModel.WindowClosing)
      .Subscribe(_ => OnPropertyChanged(nameof(FormattedText)));
  }

  private string GetFormattedText()
  {
    var percent = _cpuStatsService.GetCurrentLoadPercent().ToString(Config.NumberFormat, CultureInfo.InvariantCulture);
    return string.Format(CultureInfo.InvariantCulture, Config.StringFormat, percent);
  }
}
