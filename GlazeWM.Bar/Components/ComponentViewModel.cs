using GlazeWM.Bar.Common;

namespace GlazeWM.Bar.Components
{
  public class ComponentViewModel : ViewModelBase
  {
    protected BarViewModel _parentViewModel;

    public ComponentViewModel(BarViewModel parentViewModel)
    {
      _parentViewModel = parentViewModel;
    }
  }
}
