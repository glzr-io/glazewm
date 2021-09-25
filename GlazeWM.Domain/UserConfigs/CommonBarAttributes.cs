namespace GlazeWM.Domain.UserConfigs
{
  // TODO: Consider renaming to `SharedBarStyles`/`BarComponentConfig`.
  public class CommonBarAttributes
  {
    public string Background { get; set; } = "#101010";
    public string FontFamily { get; set; } = "Segoe UI";
    public string FontSize { get; set; } = "12";
    public string BorderColor { get; set; } = "#8192B3";

    /// <summary>
    /// Width of the border in pixels. To set a different border width for each side, specify four
    /// values (eg. "5 0 5 0"). The borders widths apply to the top, right, bottom, and left in that
    /// order.
    /// </summary>
    public string BorderWidth { get; set; } = "0 0 0 0";

    /// <summary>
    /// Padding in pixels.
    /// </summary>
    public string Padding { get; set; } = "1 6 1 6";
  }
}
