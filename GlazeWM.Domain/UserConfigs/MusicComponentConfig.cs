namespace GlazeWM.Domain.UserConfigs
{
  public class MusicComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Formatted text to display when music is not playing.
    /// </summary>
    public string LabelNotPlaying { get; set; } = "";
    /// <summary>
    /// Formatted text to display when music is not paused.
    /// </summary>
    public string LabelPaused { get; set; } = "";
    /// <summary>
    /// Formatted text to display when music is playing.
    /// </summary>
    public string LabelPlaying { get; set; } = "{song_title} - {artist_name}";
    /// <summary>
    /// The maximum length after which the song title will be truncated; if set to -1, the title will not be truncated.
    /// </summary>
    public int MaxTitleLength { get; set; } = -1;
    /// <summary>
    /// The maximum length after which the artist name will be truncated; if set to -1, the name will not be truncated.
    /// </summary>
    public int MaxArtistLength { get; set; } = -1;
  }
}
