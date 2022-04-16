using System.Collections.Generic;
using Newtonsoft.Json;

namespace GlazeWM.Domain.UserConfigs
{
  public class BarConfig
  {
    public uint Height { get; set; } = 30;
    
    public BarPosition Position { get; set; } = BarPosition.Top;

    public double Opacity { get; set; } = 1.0;

    public string Background { get; set; } = "black";

    public string Foreground { get; set; } = "white";

    public string FontFamily { get; set; } = "Segoe UI";

    public string FontSize { get; set; } = "13";

    public string BorderColor { get; set; } = "blue";

    /// <summary>
    /// Width of the border in pixels. To set a different border width for each side, specify four
    /// values (eg. "5 0 5 0"). The borders widths apply to the top, right, bottom, and left in that
    /// order.
    /// </summary>
    public string BorderWidth { get; set; } = "0";

    /// <summary>
    /// Padding in pixels.
    /// </summary>
    public string Padding { get; set; } = "0";

    [JsonProperty(ItemConverterType = typeof(BarComponentConfigConverter))]
    public List<BarComponentConfig> ComponentsLeft { get; set; } = new List<BarComponentConfig>();

    [JsonProperty(ItemConverterType = typeof(BarComponentConfigConverter))]
    public List<BarComponentConfig> ComponentsCenter { get; set; } = new List<BarComponentConfig>();

    [JsonProperty(ItemConverterType = typeof(BarComponentConfigConverter))]
    public List<BarComponentConfig> ComponentsRight { get; set; } = new List<BarComponentConfig>();
  }
}
