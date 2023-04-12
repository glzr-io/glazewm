using System;
using System.Diagnostics;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Bar.Components
{
  public class MediaComponentViewModel : ComponentViewModel
  {
    private MediaComponentConfig _config => _componentConfig as MediaComponentConfig;
    private readonly SystemMediaInformation _mediaInfo = ServiceLocator.GetRequiredService<SystemMediaInformation>();
    private string _formattedText;
    public string FormattedText
    {
      get => _formattedText;
      set
      {
        _formattedText = value;
        OnPropertyChanged(nameof(FormattedText));
      }
    }

    public MediaComponentViewModel(
      BarViewModel parentViewModel,
      MediaComponentConfig config) : base(parentViewModel, config)
    {
      _mediaInfo.CurrentMediaChanged += (_, CurrentMediaChangedEventArgs) =>
      {
        Debug.WriteLine(CurrentMediaChangedEventArgs.Title + " | " + CurrentMediaChangedEventArgs.Artist);
        FormattedText = CurrentMediaChangedEventArgs.Title + " | " + CurrentMediaChangedEventArgs.Artist;
      };

      _mediaInfo.CurrentMediaPlaybackChanged += (_, CurrentMediaPlaybackChanged) =>
      {
        Debug.WriteLine(CurrentMediaPlaybackChanged.PlaybackState.ToString());
        FormattedText = CurrentMediaPlaybackChanged.PlaybackState.ToString();
      };
    }
  }
}
