namespace GlazeWM.Domain.UserConfigs
{
  public class TilingDirectionComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Text to display in vertical mode.
    /// </summary>
    public string LabelVertical { get; set; } = "vertical";

    /// <summary>
    /// Text to display horizontal mode.
    /// </summary>
    public string LabelHorizontal { get; set; } = "horizontal";
  }
}
