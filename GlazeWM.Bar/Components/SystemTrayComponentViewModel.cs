using System.Diagnostics;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Bussing;
using ManagedShell;

namespace GlazeWM.Bar.Components
{
  public class SystemTrayComponentViewModel : ComponentViewModel
  {
    // private SystemTrayComponentConfig _config => _componentConfig as SystemTrayComponentConfig;

    // public string Text => _config.Text;

    public SystemTrayComponentViewModel(
      BarViewModel parentViewModel,
      SystemTrayComponentConfig config) : base(parentViewModel, config)
    {
      Debug.WriteLine("--");
    }
  }
}