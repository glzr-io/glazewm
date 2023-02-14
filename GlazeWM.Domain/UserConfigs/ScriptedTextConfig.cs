using System;

namespace GlazeWM.Domain.UserConfigs;

public class ScriptedTextConfig : BarComponentConfig
{
  public string ScriptPath { get; set; }
  public string Label { get; set; } = "";
  public string ScriptArgs { get; set; } = "";
  public int IntervalMs { get; set; } = 5 * 1000;
  public string OutputType { get; set; } = "json";
}
