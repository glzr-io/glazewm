
using System;
using Windows.Media.Control;

namespace GlazeWM.Infrastructure.WindowsApi
{
  public class SystemMediaInformation
  {
    private readonly GlobalSystemMediaTransportControlsSessionManager _gsmtcsManager;
    private GlobalSystemMediaTransportControlsSession _currentSession;

    public event EventHandler<CurrentMediaPlaybackChangedEventArgs> CurrentMediaPlaybackChanged;
    public event EventHandler<CurrentMediaChangedEventArgs> CurrentMediaChanged;

    public SystemMediaInformation()
    {
      _gsmtcsManager = GlobalSystemMediaTransportControlsSessionManager
        .RequestAsync()
        .GetAwaiter()
        .GetResult();
      _gsmtcsManager.CurrentSessionChanged += OnCurrentSessionChanged;
      _currentSession = _gsmtcsManager.GetCurrentSession();
      _currentSession.MediaPropertiesChanged += OnMediaPropertiesChanged;
      var mediaProps = _currentSession
        .TryGetMediaPropertiesAsync()
        .GetAwaiter()
        .GetResult();
      CurrentMediaChanged?.Invoke(
        this, new CurrentMediaChangedEventArgs
        {
          Title = mediaProps.Title,
          AlbumTitle = mediaProps.AlbumTitle,
          Artist = mediaProps.Artist
        }
      );
      var currentState = _currentSession.GetPlaybackInfo().PlaybackStatus;
      CurrentMediaPlaybackChanged?.Invoke(
        this, new CurrentMediaPlaybackChangedEventArgs { PlaybackState = currentState }
      );
    }

    private void OnCurrentSessionChanged(GlobalSystemMediaTransportControlsSessionManager sender, CurrentSessionChangedEventArgs args)
    {
      _currentSession.MediaPropertiesChanged -= OnMediaPropertiesChanged;
      _currentSession = _gsmtcsManager.GetCurrentSession();
      _currentSession.MediaPropertiesChanged += OnMediaPropertiesChanged;
      var mediaProps = _currentSession
        .TryGetMediaPropertiesAsync()
        .GetAwaiter()
        .GetResult();
      CurrentMediaChanged?.Invoke(
        this, new CurrentMediaChangedEventArgs
        {
          Title = mediaProps.Title,
          AlbumTitle = mediaProps.AlbumTitle,
          Artist = mediaProps.Artist
        }
      );
    }

    private void OnMediaPropertiesChanged(GlobalSystemMediaTransportControlsSession sender, MediaPropertiesChangedEventArgs args)
    {
      var newState = _currentSession.GetPlaybackInfo().PlaybackStatus;
      CurrentMediaPlaybackChanged?.Invoke(
        this, new CurrentMediaPlaybackChangedEventArgs { PlaybackState = newState }
      );
    }
  }
}
