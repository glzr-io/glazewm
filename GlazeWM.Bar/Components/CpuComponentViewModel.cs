using System;
using System.Globalization;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Bar.Components
{
  public class CpuComponentViewModel : ComponentViewModel
  {
    private readonly CpuComponentConfig _config;
    private readonly CpuStatsService _cpuStatsService;

    public string FormattedText => GetFormattedText();

    public CpuComponentViewModel(
      BarViewModel parentViewModel,
      CpuComponentConfig config) : base(parentViewModel, config)
    {
      _config = config;
      _cpuStatsService = ServiceLocator.GetRequiredService<CpuStatsService>();

      Observable
        .Interval(TimeSpan.FromMilliseconds(_config.RefreshIntervalMs))
        .TakeUntil(_parentViewModel.WindowClosing)
        .Subscribe(_ => OnPropertyChanged(nameof(FormattedText)));
    }

    private string GetFormattedText()
    {
      var values = _cpuStatsService.GetMeasurement(_config.Counter);
      values.DivideBy(_config.DivideBy);

      var percent = (values.CurrentValue / values.MaxValue * 100).ToString(_config.PercentFormat, CultureInfo.InvariantCulture);
      var curValue = values.CurrentValue.ToString(_config.CurrentValueFormat, CultureInfo.InvariantCulture);
      var maxValue = values.MaxValue.ToString(_config.MaxValueFormat, CultureInfo.InvariantCulture);

      return string.Format(CultureInfo.InvariantCulture, _config.StringFormat, percent, curValue, maxValue);
    }
  }
}
