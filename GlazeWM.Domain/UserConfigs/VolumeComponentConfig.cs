namespace GlazeWM.Domain.UserConfigs
{
  public class VolumeComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Default label.
    /// </summary>
    public string Label { get; set; } = "{volume_level}%";
    /// <summary>
    /// Label for low volume.
    /// </summary>
    public string LabelLow { get; set; } = "<attr ff='GlazeWM.App.Resources#Material Icons'></attr>{volume_level}%";
    /// <summary>
    /// Label for medium volume.
    /// </summary>
    public string LabelMedium { get; set; } = "<attr ff='GlazeWM.App.Resources#Material Icons'></attr>{volume_level}%";
    /// <summary>
    /// Label for high volume.
    /// </summary>
    public string LabelHigh { get; set; } = "<attr ff='GlazeWM.App.Resources#Material Icons'></attr>{volume_level}%";
    /// <summary>
    /// Label for volume mute.
    /// </summary>
    public string LabelMute { get; set; } = "<attr ff='GlazeWM.App.Resources#Material Icons'></attr>{volume_level}%";
  }

}
