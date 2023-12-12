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
    public struct SongSession
    {
      public SongSession(GlobalSystemMediaTransportControlsSessionPlaybackStatus musicStatus, string songTitle, string artistName)
      {
        SongTitle = songTitle;
        ArtistName = artistName;
        MusicStatus = musicStatus;
      }
      public string SongTitle { get; set; }
      public string ArtistName { get; set; }
      public GlobalSystemMediaTransportControlsSessionPlaybackStatus MusicStatus { get; set; }
    }
    private readonly Dictionary<string, SongSession> songSessionDict;
    public MusicComponentViewModel(
      BarViewModel parentViewModel,
      MusicComponentConfig config) : base(parentViewModel, config)
    {
      songSessionDict = new Dictionary<string, SongSession>();
      _config = config;
      mediaManager = new MediaManager();
      mediaManager.OnAnyMediaPropertyChanged += (session, args) => MusicTitleChanged(session.Id, args.Artist, args.Title);
      mediaManager.OnAnyPlaybackStateChanged += (session, args) => PlaybackStateChanged(session.Id, args.PlaybackStatus);
      mediaManager.OnAnySessionOpened += (session) => OpenedSession(session.Id);
      mediaManager.OnAnySessionClosed += (session) => ClosedSession(session.Id);
      mediaManager.Start();
    }
    private string GetLabel(GlobalSystemMediaTransportControlsSessionPlaybackStatus musicStatus)
    {
      return musicStatus switch
      {
        GlobalSystemMediaTransportControlsSessionPlaybackStatus.Playing => _config.LabelPlaying,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus.Paused => _config.LabelPaused,
        _ => _config.LabelNotPlaying,
      };
    }
    private void MusicTitleChanged(string sessionId, string artistName, string songTitle)
    {
      var session = songSessionDict[sessionId];
      session.ArtistName = artistName;
      session.SongTitle = songTitle;
      songSessionDict[sessionId] = session;
      Label = CreateLabel();
    }
    private void PlaybackStateChanged(string sessionId, GlobalSystemMediaTransportControlsSessionPlaybackStatus status)
    {
      var session = songSessionDict[sessionId];
      session.MusicStatus = status;
      songSessionDict[sessionId] = session;
      Label = CreateLabel();
    }
    private void OpenedSession(string sessionId)
    {
      var session = new SongSession(GlobalSystemMediaTransportControlsSessionPlaybackStatus.Opened, "", "");
      songSessionDict.Add(sessionId, session);
    }
    private void ClosedSession(string sessionId)
    {
      songSessionDict.Remove(sessionId);
      Label = CreateLabel();
    }
    private LabelViewModel CreateLabel()
    {
      var label = _config.LabelNotPlaying;
      var variableDictionary = new Dictionary<string, Func<string>>()
      {
        {
          "song_title", () => ""
        },
        {
          "artist_name", () => ""
        }
      };

      foreach (var session in songSessionDict)
      {
        if(GetLabel(session.Value.MusicStatus) == _config.LabelPaused && label != _config.LabelPlaying)
        {
          label = _config.LabelPaused;
          variableDictionary["song_title"] = () => session.Value.SongTitle;
          variableDictionary["artist_name"] = () => session.Value.ArtistName;
        }
        else if (GetLabel(session.Value.MusicStatus) == _config.LabelPlaying)
        {
          label = _config.LabelPlaying;
          variableDictionary["song_title"] = () => session.Value.SongTitle;
          variableDictionary["artist_name"] = () => session.Value.ArtistName;
        }
      }
      return XamlHelper.ParseLabel(
        label,
        variableDictionary,
        this
      );
    }
  }
}
