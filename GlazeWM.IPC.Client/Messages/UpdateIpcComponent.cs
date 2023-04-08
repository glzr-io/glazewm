namespace GlazeWM.IPC.Client.Messages;

/// <summary>
/// Updates a given IPC-powered label.
/// The settings here use the same logic as the config; and thus settings can be specified in multiple ways.
/// </summary>
public struct UpdateIpcComponent
{
  /// <summary>
  /// Text to use for this component.
  /// </summary>
  public string? Text { get; set; }
  
  /// <summary>
  /// Margin to use for the component.
  /// </summary>
  public string? Margin { get; set; }

  /// <summary>
  /// Background of the bar component. Use transparent as a default, since falling back
  /// to bar background config looks weird with nested semi-transparent backgrounds.
  /// </summary>
  public string? Background { get; set; }

  /// <summary>
  /// Fallback to bar foreground config if unset.
  /// </summary>
  public string? Foreground { get; set; }

  /// <summary>
  /// Fallback to bar font family config if unset.
  /// </summary>
  public string? FontFamily { get; set; }

  /// <summary>
  /// Fallback to bar font weight config if unset.
  /// </summary>
  public string? FontWeight { get; set; }

  /// <summary>
  /// Fallback to bar font size config if unset.
  /// </summary>
  public string? FontSize { get; set; }

  /// <summary>
  /// Colour of the border
  /// </summary>
  public string? BorderColor { get; set; }

  /// <summary>
  /// Radius of the border in pixels.
  /// </summary>
  public string? BorderRadius { get; set; }

  /// <summary>
  /// Width of the border in pixels. To set a different border width for each side, specify four
  /// values (eg. "5 0 5 0"). The borders widths apply to the top, right, bottom, and left in that
  /// order.
  /// </summary>
  public string? BorderWidth { get; set; }

  /// <summary>
  /// Padding in pixels.
  /// </summary>
  public string? Padding { get; set; }
}
