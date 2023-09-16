namespace GlazeWM.Domain.UserConfigs
{
  public class NetworkComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Label for wifi at 0% RSSI.
    /// </summary>
    public string LabelWifiStrength0 { get; set; } = "<attr ff='pack://application:,,,/Resources/#Material Icons'></attr>";
    /// <summary>
    /// Label for wifi at 25% RSSI.
    /// </summary>
    public string LabelWifiStrength25 { get; set; } = "<attr ff='pack://application:,,,/Resources/#Material Icons'></attr>";
    /// <summary>
    /// Label for wifi at 50% RSSI.
    /// </summary>
    public string LabelWifiStrength50 { get; set; } = "<attr ff='pack://application:,,,/Resources/#Material Icons'></attr>";
    /// <summary>
    /// Label for wifi at 75% RSSI.
    /// </summary>
    public string LabelWifiStrength75 { get; set; } = "<attr ff='pack://application:,,,/Resources/#Material Icons'></attr>";
    /// <summary>
    /// Label for wifi at 100% RSSI.
    /// </summary>
    public string LabelWifiStrength100 { get; set; } = "<attr ff='pack://application:,,,/Resources/#Material Icons'></attr>";
    /// <summary>
    /// Label for ethernet connection.
    /// </summary>
    public string LabelEthernet { get; set; } = "<attr ff='pack://application:,,,/Resources/#Material Icons'></attr>";
    /// <summary>
    /// Label for connection to the internet.
    /// </summary>
    public string LabelNoInternet { get; set; } = "<attr ff='pack://application:,,,/Resources/#Material Icons'></attr>";
  }
}
