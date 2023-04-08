namespace GlazeWM.Infrastructure.Services.IpcMessages;

/// <summary>
/// Updates a label with a specific ID remotely.
/// Values that are null are interpreted as "don't change".
/// Non-null values are parsed using the same rules as config file and updated accordingly.
/// </summary>
public class UpdateLabel
{
  /// <summary>
  /// Unique identifier of the label to update.
  /// </summary>
  public string LabelId { get; set; }

  /// <summary>
  /// Size of the item margin.
  /// </summary>
  public string Margin { get; set; }

  /// <summary>
  /// Background of the bar component. Use transparent as a default, since falling back
  /// to bar background config looks weird with nested semi-transparent backgrounds.
  /// </summary>
  public string Background { get; set; }

  /// <summary>
  /// Fallback to bar foreground config if unset.
  /// </summary>
  public string Foreground { get; set; }

  /// <summary>
  /// Fallback to bar font family config if unset.
  /// </summary>
  public string FontFamily { get; set; }

  /// <summary>
  /// Fallback to bar font weight config if unset.
  /// </summary>
  public string FontWeight { get; set; }

  /// <summary>
  /// Fallback to bar font size config if unset.
  /// </summary>
  public string FontSize { get; set; }

  /// <summary>
  /// Colour of the border.
  /// </summary>
  public string BorderColor { get; set; }

  /// <summary>
  /// Radius of the border in pixels.
  /// </summary>
  public string BorderRadius { get; set; }

  /// <summary>
  /// Width of the border in pixels. To set a different border width for each side, specify four
  /// values (eg. "5 0 5 0"). The borders widths apply to the top, right, bottom, and left in that
  /// order.
  /// </summary>
  public string BorderWidth { get; set; }

  /// <summary>
  /// Padding in pixels.
  /// </summary>
  public string Padding { get; set; }
}
