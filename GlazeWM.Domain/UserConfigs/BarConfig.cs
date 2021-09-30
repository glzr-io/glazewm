using System.Collections.Generic;
using Newtonsoft.Json;

namespace GlazeWM.Domain.UserConfigs
{
  public class BarConfig : CommonBarAttributes
  {
    public uint Height { get; set; } = 25;

    public double Opacity { get; set; } = 0.9;

    [JsonProperty(ItemConverterType = typeof(BarComponentConfigConverter))]
    public List<BarComponentConfig> ComponentsLeft { get; set; } = new List<BarComponentConfig>();

    [JsonProperty(ItemConverterType = typeof(BarComponentConfigConverter))]
    public List<BarComponentConfig> ComponentsCenter { get; set; } = new List<BarComponentConfig>();

    [JsonProperty(ItemConverterType = typeof(BarComponentConfigConverter))]
    public List<BarComponentConfig> ComponentsRight { get; set; } = new List<BarComponentConfig>();
  }
}
