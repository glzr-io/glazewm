using Windows.Media.Control;

namespace GlazeWM.Infrastructure.WindowsApi
{
  public class CurrentMediaPlaybackChangedEventArgs
  {
    public GlobalSystemMediaTransportControlsSessionPlaybackStatus PlaybackState { get; set; }
  }
}
