namespace GlazeWM.Domain.UserConfigs
{
  public class NetworkComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Text to display.
    /// </summary>
    /// <summary>
    /// Text to display.
    /// </summary>
    public string LabelWifiStrength0 { get; set; } = "  ";
    public string LabelWifiStrength25 { get; set; } = "  ";
    public string LabelWifiStrength50 { get; set; } = "  ";
    public string LabelWifiStrength75 { get; set; } = "  ";
    public string LabelWifiStrength100 { get; set; } = "  ";
    public string LabelEthernet { get; set; } = "  ";
    public string LabelNoInternet { get; set; } = "  ";
  }
}
