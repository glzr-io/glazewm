namespace GlazeWM.Domain.UserConfigs
{
  public class TilingDirectionComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Text to display in vertical mode.
    /// </summary>
    public string TextVertical { get; set; } = "vertical";

    /// <summary>
    /// Text to display horizontal mode.
    /// </summary>
    public string TextHorizontal { get; set; } = "horizontal";
  }
}
