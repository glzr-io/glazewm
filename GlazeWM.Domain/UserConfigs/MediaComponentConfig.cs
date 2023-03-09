namespace GlazeWM.Domain.UserConfigs
{
  public class MediaComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Label for music playing
    /// </summary>
    public string LabelMusicPlaying { get; set; } = "";

    /// <summary>
    /// Label for music paused
    /// </summary>
    public string LabelMusicPaused { get; set; } = "";

    public MediaComponentConfig()
    {
      FontFamily = "pack://application:,,,/Resources/#Font Awesome 6 Free Solid";
    }
  }
}
