using System;
using System.Collections.Generic;
using System.Globalization;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Bar.Components
{
  public class MemoryComponentViewModel : ComponentViewModel
  {
    private readonly MemoryComponentConfig _config;
    private readonly MemoryStatsService _memoryStatsService;

    private LabelViewModel _label;
    public LabelViewModel Label
    {
      get => _label;
      protected set => SetField(ref _label, value);
    }

    public MemoryComponentViewModel(
      BarViewModel parentViewModel,
      MemoryComponentConfig config) : base(parentViewModel, config)
    {
      _config = config;
      _memoryStatsService = ServiceLocator.GetRequiredService<MemoryStatsService>();

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
          () => _memoryStatsService.GetMemoryUsage().ToString("0", CultureInfo.InvariantCulture)
        }
      };

      return XamlHelper.ParseLabel(_config.Label, variableDictionary, this);
    }
  }
}
