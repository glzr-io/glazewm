namespace GlazeWM.Domain.UserConfigs
{
  /// <summary>
  /// <para>
  /// A component to display information about the battery of the device, namely the battery level and
  /// whether the device is connected to power or is in power saving mode.
  /// </para>
  /// <para>
  /// There are 3 labels that are used depending on the situation, and the variable
  /// <c>{battery_level}</c> is replaced with the current battery level of the device.
  /// </para>
  /// <para>
  /// When the device does not have a dedicated battery, the battery level is displayed
  /// as 100% at all times.
  /// </para>
  /// </summary>
  public class BatteryComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Formatted text to display when the device is draining battery power and battery level is low.
    /// </summary>
    public string LabelDrainingLow { get; set; } = "{battery_level}% (low)";
    /// <summary>
    /// Formatted text to display when the device is draining battery power and battery level is medium.
    /// </summary>
    public string LabelDrainingMedium { get; set; } = "{battery_level}% (medium)";
    /// <summary>
    /// Formatted text to display when the device is draining battery power and battery level is high.
    /// </summary>
    public string LabelDrainingHigh { get; set; } = "{battery_level}% (high)";
    /// <summary>
    /// Formatted text to display when the device is in power saving mode and battery level is low.
    /// </summary>
    public string LabelPowerSaverLow { get; set; } = "{battery_level}% (power saver) (low)";
    /// <summary>
    /// Formatted text to display when the device is in power saving mode and battery level is medium.
    /// </summary>
    public string LabelPowerSaverMedium { get; set; } = "{battery_level}% (power saver) (medium)";
    /// <summary>
    /// Formatted text to display when the device is in power saving mode and battery level is high.
    /// </summary>
    public string LabelPowerSaverHigh { get; set; } = "{battery_level}% (power saver) (high)";
    /// <summary>
    /// Formatted text to display when the device is connected to power and battery level is low.
    /// </summary>
    public string LabelChargingLow { get; set; } = "{battery_level}% (charging) (low)";
    /// <summary>
    /// Formatted text to display when the device is connected to power and battery level is medium.
    /// </summary>
    public string LabelChargingMedium { get; set; } = "{battery_level}% (charging) (medium)";
    /// <summary>
    /// Formatted text to display when the device is connected to power and battery level is high.
    /// </summary>
    public string LabelChargingHigh { get; set; } = "{battery_level}% (charging) (high)";
  }
}
