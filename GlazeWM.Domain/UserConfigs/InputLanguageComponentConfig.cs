namespace GlazeWM.Domain.UserConfigs
{
  public class InputLanguageComponentConfig : BarComponentConfig
  {
    public string Label { get; set; } = "Input: {input_language}";
    public int RefreshIntervalMs { get; set; } = 1000;
  }
}
