namespace GlazeWM.Domain.UserConfigs
{
  public class FocusBordersConfig
  {
    /// <summary>
    /// Border of the focused window.
    /// </summary>
    public FocusBorder Active { get; set; } = new() { Enabled = false, Color = "#0000ff" };

    /// <summary>
    /// Border of non-focused windows.
    /// </summary>
    public FocusBorder Inactive { get; set; } = new() { Enabled = false, Color = "#ff0000" };
  }

  public class FocusBorder
  {
    /// <summary>
    /// Should the default transparent border be used.
    /// </summary>
    public bool Enabled;

    /// <summary>
    /// Border color of window.
    /// </summary>
    public string Color;
  }
}
