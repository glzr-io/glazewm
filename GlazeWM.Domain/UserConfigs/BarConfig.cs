using System.Collections.Generic;
using System.Linq;

namespace GlazeWM.Domain.UserConfigs
{
  public class BarConfig : CommonBarAttributes
  {
    public uint Height { get; set; } = 25;

    public double Opacity { get; set; } = 0.9;

    List<BarComponentConfig> _componentsLeft;
    public List<BarComponentConfig> ComponentsLeft
    {
      get { return _componentsLeft; }
      set { _componentsLeft = value.Select(configs => configs.Upcast()).ToList(); }
    }

    public List<BarComponentConfig> ComponentsCenter { get; set; } = new List<BarComponentConfig>();

    public List<BarComponentConfig> ComponentsRight { get; set; } = new List<BarComponentConfig>();
  }
}
