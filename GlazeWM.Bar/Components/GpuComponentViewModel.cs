using System;
using System.Globalization;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Bar.Components
{
  public class GpuComponentViewModel : ComponentViewModel
  {
    private GpuComponentConfig Config => _componentConfig as GpuComponentConfig;
    private readonly GpuStatsService _gpuStatsService;

    public string FormattedText => GetFormattedText();

    public GpuComponentViewModel(BarViewModel parentViewModel, GpuComponentConfig config) : base(parentViewModel, config)
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
}
