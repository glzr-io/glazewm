namespace GlazeWM.Domain.UserConfigs
{
  public class TextFileComponentConfig : BarComponentConfig
  {
    public string FilePath { get; set; }

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
