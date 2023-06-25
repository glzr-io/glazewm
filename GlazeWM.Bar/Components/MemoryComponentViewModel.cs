using System;
using System.Globalization;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Bar.Components;

public class MemoryComponentViewModel : ComponentViewModel
{
  private MemoryComponentConfig Config => _componentConfig as MemoryComponentConfig;
  private readonly MemoryStatsService _gpuStatsService;

  public string FormattedText => GetFormattedText();

  public MemoryComponentViewModel(BarViewModel parentViewModel, MemoryComponentConfig config) : base(parentViewModel, config)
  {
    _gpuStatsService = ServiceLocator.GetRequiredService<MemoryStatsService>();
    Observable
      .Interval(TimeSpan.FromMilliseconds(Config.RefreshIntervalMs))
      .TakeUntil(_parentViewModel.WindowClosing)
      .Subscribe(_ => OnPropertyChanged(nameof(FormattedText)));
  }

  private string GetFormattedText()
  {
    var values = _gpuStatsService.GetMeasurement(Config.Counter);
    values.DivideBy(Config.DivideBy);

    var percent = ((values.CurrentValue / values.MaxValue) * 100).ToString(Config.PercentFormat, CultureInfo.InvariantCulture);
    var curValue = values.CurrentValue.ToString(Config.CurrentValueFormat, CultureInfo.InvariantCulture);
    var maxValue = values.MaxValue.ToString(Config.MaxValueFormat, CultureInfo.InvariantCulture);

    return string.Format(CultureInfo.InvariantCulture, Config.StringFormat, percent, curValue, maxValue);
  }
}
