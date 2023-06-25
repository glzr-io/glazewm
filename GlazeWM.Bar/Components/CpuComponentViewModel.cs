using System;
using System.Globalization;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Bar.Components;

public class CpuComponentViewModel : ComponentViewModel
{
  private CpuComponentConfig Config => _componentConfig as CpuComponentConfig;
  private readonly CpuStatsService _cpuStatsService;

  public string FormattedText => GetFormattedText();

  public CpuComponentViewModel(BarViewModel parentViewModel, CpuComponentConfig config) : base(parentViewModel, config)
  {
    _cpuStatsService = ServiceLocator.GetRequiredService<CpuStatsService>();

    Observable
      .Interval(TimeSpan.FromMilliseconds(Config.RefreshIntervalMs))
      .TakeUntil(_parentViewModel.WindowClosing)
      .Subscribe(_ => OnPropertyChanged(nameof(FormattedText)));
  }

  private string GetFormattedText()
  {
    var values = _cpuStatsService.GetMeasurement(Config.Counter);
    values.DivideBy(Config.DivideBy);

    var percent = ((values.CurrentValue / values.MaxValue) * 100).ToString(Config.PercentFormat, CultureInfo.InvariantCulture);
    var curValue = values.CurrentValue.ToString(Config.CurrentValueFormat, CultureInfo.InvariantCulture);
    var maxValue = values.MaxValue.ToString(Config.MaxValueFormat, CultureInfo.InvariantCulture);

    return string.Format(CultureInfo.InvariantCulture, Config.StringFormat, percent, curValue, maxValue);
  }
}
