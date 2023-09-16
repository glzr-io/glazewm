namespace GlazeWM.Bar.Components
{
  public class LabelSpan
  {
    public string Text { get; }
    public string Background { get; }
    public string Foreground { get; }
    public string FontFamily { get; }
    public string FontWeight { get; }
    public string FontSize { get; }

    public LabelSpan(
      string text,
      string background,
      string foreground,
      string fontFamily,
      string fontWeight,
      string fontSize)
    {
      Text = text;
      Background = background;
      Foreground = foreground;
      FontFamily = fontFamily;
      FontWeight = fontWeight;
      FontSize = fontSize;
    }
  }
}
