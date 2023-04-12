namespace GlazeWM.Domain.UserConfigs
{
  public class VolumeComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Icon for low volume.
    /// </summary>
    public string LabelVolumeLow { get; set; } = "";
    /// <summary>
    /// Icon for medium volume.
    /// </summary>
    public string LabelVolumeMed { get; set; } = "";
    /// <summary>
    /// Icon for high volume.
    /// </summary>
    public string LabelVolumeHigh { get; set; } = "";
    /// <summary>
    /// Icon for volume mute.
    /// </summary>
    public string LabelVolumeMute { get; set; } = "";
    /// <summary>
    /// Sets default icon font if one isn't specified.
    /// </summary>
    public VolumeComponentConfig()
    {
      FontFamily = "pack://application:,,,/Resources/#Material Icons";
    }
  }

}
