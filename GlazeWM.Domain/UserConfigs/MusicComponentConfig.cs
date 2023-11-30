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
  }
}
