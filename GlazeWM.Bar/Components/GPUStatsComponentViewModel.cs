using System;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Bar.Components
{
  public class GPUStatsComponentViewModel : ComponentViewModel
  {
    private GPUStatsComponentConfig _config => _componentConfig as GPUStatsComponentConfig;
    public string FormattedText => GetSystemStats();
    private readonly GPUStatsService _gpuStatsService = new();

    private string GetSystemStats()
    {
      return _config.LabelGPU + _gpuStatsService.GetCurrentUtilization().ToString("0.") + "%";
    }
    public GPUStatsComponentViewModel(
      BarViewModel parentViewModel,
      GPUStatsComponentConfig config) : base(parentViewModel, config)
    {
      Observable.Interval(TimeSpan.FromSeconds(2))
        .Subscribe(_ => OnPropertyChanged(nameof(FormattedText)));
    }
  }
}
