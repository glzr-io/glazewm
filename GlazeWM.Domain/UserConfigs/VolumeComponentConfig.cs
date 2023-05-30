namespace GlazeWM.Domain.UserConfigs
{
  public class VolumeComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Formatted text to display current volume level.
    /// </summary>
    public string Label { get; set; } = "<attr ff='pack://application:,,,/Resources/#Material Icons'>{volume_icon}</attr>{volume_level}%";
    /// <summary>
    /// Icon for low volume.
    /// </summary>
    public string IconVolumeLow { get; set; } = "";
    /// <summary>
    /// Icon for medium volume.
    /// </summary>
    public string IconVolumeMed { get; set; } = "";
    /// <summary>
    /// Icon for high volume.
    /// </summary>
    public string IconVolumeHigh { get; set; } = "";
    /// <summary>
    /// Icon for volume mute.
    /// </summary>
    public string IconVolumeMute { get; set; } = "";
    /// <summary>
    /// Sets default icon font if one isn't specified.
    /// </summary>
  }

}
