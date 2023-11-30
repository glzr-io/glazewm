using System;
using System.Collections.Generic;
using GlazeWM.Domain.UserConfigs;
using Windows.Media.Control;
using WindowsMediaController;

namespace GlazeWM.Bar.Components
{
  public class MusicComponentViewModel : ComponentViewModel
  {
    private readonly MusicComponentConfig _config;
    private static MediaManager mediaManager;
    private LabelViewModel _label;
    public LabelViewModel Label
    {
      get => _label;
      protected set => SetField(ref _label, value);
    }
    public string songTitle;
    public string artistName;
    public GlobalSystemMediaTransportControlsSessionPlaybackStatus musicStatus;
    public MusicComponentViewModel(
      BarViewModel parentViewModel,
      MusicComponentConfig config) : base(parentViewModel, config)
    {
      _config = config;
      mediaManager = new MediaManager();
      mediaManager.OnAnyMediaPropertyChanged += (_, args) => MusicTitleChanged(args.Artist, args.Title);
      mediaManager.OnAnyPlaybackStateChanged += (_, args) => PlaybackStateChanged(args.PlaybackStatus);
      mediaManager.Start();
    }

    private string GetLabel()
    {
      return musicStatus switch
      {
        GlobalSystemMediaTransportControlsSessionPlaybackStatus.Playing => _config.LabelPlaying,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus.Paused => _config.LabelPaused,
        _ => _config.LabelNotPlaying,
      };
    }
    private void MusicTitleChanged(string name, string title)
    {
      songTitle = title;
      artistName = name;
      Label = CreateLabel();
    }
    private void PlaybackStateChanged(GlobalSystemMediaTransportControlsSessionPlaybackStatus status)
    {
      musicStatus = status;
      Label = CreateLabel();
    }
    private LabelViewModel CreateLabel()
    {
      var variablesDictionary = new Dictionary<string, Func<string>>()
      {
          {"song_title", () => songTitle},
          {"artist_name", () => artistName}
      };

      return XamlHelper.ParseLabel(
        GetLabel(),
        variablesDictionary,
        this
      );
    }
  }
}
