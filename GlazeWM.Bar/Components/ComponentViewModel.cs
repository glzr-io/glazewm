using GlazeWM.Bar.Common;
using GlazeWM.Domain.UserConfigs;

namespace GlazeWM.Bar.Components
{
  public class ComponentViewModel : ViewModelBase
  {
    protected readonly BarViewModel _parentViewModel;
    protected readonly BarComponentConfig _componentConfig;

    public string Background => _componentConfig.Background ?? _parentViewModel.Background;
    public string Foreground => _componentConfig.Foreground ?? _parentViewModel.Foreground;
    public string FontFamily => _componentConfig.FontFamily ?? _parentViewModel.FontFamily;
    public string FontWeight => _componentConfig.FontWeight ?? _parentViewModel.FontWeight;
    public string FontSize => _componentConfig.FontSize ?? _parentViewModel.FontSize;
    public string BorderColor => _componentConfig.BorderColor;
    public string BorderWidth => BarService.ShorthandToXamlProperty(_componentConfig.BorderWidth);
    public string Padding => BarService.ShorthandToXamlProperty(_componentConfig.Padding);
    public string Margin => BarService.ShorthandToXamlProperty(_componentConfig.Margin);

    public ComponentViewModel(BarViewModel parentViewModel, BarComponentConfig baseConfig)
    {
      _parentViewModel = parentViewModel;
      _componentConfig = baseConfig;
    }
  }
}
