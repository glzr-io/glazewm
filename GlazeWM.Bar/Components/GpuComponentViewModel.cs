using System;
using System.Collections.Generic;
using System.Globalization;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Bar.Components
{
  public class GpuComponentViewModel : ComponentViewModel
  {
    private readonly GpuComponentConfig _config;
    private readonly GpuStatsService _gpuStatsService;

    private LabelViewModel _label;
    public LabelViewModel Label
    {
      get => _label;
      protected set => SetField(ref _label, value);
    }

    public GpuComponentViewModel(
      BarViewModel parentViewModel,
      GpuComponentConfig config) : base(parentViewModel, config)
    {
      _config = config;
      _gpuStatsService = ServiceLocator.GetRequiredService<GpuStatsService>();

      Observable.Timer(
        TimeSpan.Zero,
        TimeSpan.FromMilliseconds(_config.RefreshIntervalMs)
      )
        .TakeUntil(_parentViewModel.WindowClosing)
        .Subscribe((_) => Label = CreateLabel());
    }

    private LabelViewModel CreateLabel()
    {
      var variableDictionary = new Dictionary<string, Func<string>>()
      {
        {
          "percent_usage",
          () =>
            _gpuStatsService
              .GetAverageLoadPercent(GpuPerformanceCategoryFlags.Graphics)
              .ToString("0", CultureInfo.InvariantCulture)
        }
      };

      return XamlHelper.ParseLabel(_config.Label, variableDictionary, this);
    }
  }
}
