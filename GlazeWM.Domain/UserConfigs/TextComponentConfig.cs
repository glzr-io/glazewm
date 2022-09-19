namespace GlazeWM.Domain.UserConfigs
{
  public class TextComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Text to display.
    /// </summary>
    public string Text { get; set; } = "Hello world!";

    /// <summary>
    /// Command to invoke on left-click.
    /// </summary>
    public string LeftClickCommand { get; set; }

    /// <summary>
    /// Command to invoke on right-click.
    /// </summary>
    public string RightClickCommand { get; set; }
  }
}
