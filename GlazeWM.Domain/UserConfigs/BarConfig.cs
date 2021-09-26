using System.Collections.Generic;

namespace GlazeWM.Domain.UserConfigs
{
  public class BarConfig : CommonBarAttributes
  {
    public uint Height { get; set; } = 25;

    public double Opacity { get; set; } = 0.9;

    public List<BarComponentConfig> ComponentsLeft = CreateMockComponentConfig();
    public List<BarComponentConfig> ComponentsCenter = new List<BarComponentConfig>();
    public List<BarComponentConfig> ComponentsRight = CreateMockComponentConfig();

    private static List<BarComponentConfig> CreateMockComponentConfig()
    {
      var sampleConfig1 = new WorkspacesComponentConfig()
      {
        Type = "workspaces",
      };

      var sampleConfig2 = new ClockComponentConfig()
      {
        Type = "clock",
      };

      return new List<BarComponentConfig> { sampleConfig1, sampleConfig2 };
    }
  }
}
