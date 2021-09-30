using GlazeWM.Bar.Common;
using GlazeWM.Domain.UserConfigs;

namespace GlazeWM.Bar.Components
{
  public class ComponentViewModel : ViewModelBase
  {
    protected readonly BarViewModel _parentViewModel;
    protected readonly BarComponentConfig _config;

    public ComponentViewModel(BarViewModel parentViewModel, BarComponentConfig config)
    {
      _parentViewModel = parentViewModel;
      _config = config;
    }
  }
}
