using System;

namespace GlazeWM.Domain.UserConfigs;

public class ScriptedTextConfig : BarComponentConfig
{
  public string ScriptPath { get; set; }
  public string Format { get; set; } = "";
  public string Args { get; set; } = "";
  public int IntervalMs { get; set; } = 5 * 1000;
}
