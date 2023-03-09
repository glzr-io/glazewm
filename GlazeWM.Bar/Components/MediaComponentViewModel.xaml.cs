using System;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;

namespace GlazeWM.Bar.Components
{
  public class MediaComponentViewModel : ComponentViewModel
  {
    private MediaComponentConfig _config => _componentConfig as MediaComponentConfig;
    public string Text => FormatLabel();
    private string FormatLabel()
    {
      return _config.LabelMusicPlaying;
    }

    public MediaComponentViewModel(
      BarViewModel parentViewModel,
      MediaComponentConfig config) : base(parentViewModel, config)
    {
      Observable
        .Interval(TimeSpan.FromSeconds(10))
        .TakeUntil(_parentViewModel.WindowClosing)
        .Subscribe(_ => OnPropertyChanged(nameof(Text)));

    }
  }
}
