using System;
using System.Globalization;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Bar.Components;

public class GpuPercentComponentViewModel : ComponentViewModel
{
  private GpuPercentComponentConfig Config => _componentConfig as GpuPercentComponentConfig;
  private GpuStatsService _gpuStatsService;
  
  public string FormattedText => GetFormattedText();

  public GpuPercentComponentViewModel(BarViewModel parentViewModel, GpuPercentComponentConfig config) : base(parentViewModel, config)
  {
    _gpuStatsService = ServiceLocator.GetRequiredService<GpuStatsService>();
    Observable
      .Interval(TimeSpan.FromMilliseconds(Config.RefreshIntervalMs))
      .TakeUntil(_parentViewModel.WindowClosing)
      .Subscribe(_ => OnPropertyChanged(nameof(FormattedText)));
  }

  private string GetFormattedText()
  {
    var percent = _gpuStatsService.GetAverageLoadPercent(Config.Flags).ToString(Config.NumberFormat, CultureInfo.InvariantCulture);
    return string.Format(CultureInfo.InvariantCulture, Config.StringFormat, percent);
  }
}
