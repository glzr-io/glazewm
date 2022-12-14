using GlazeWM.Bar.Common;
using GlazeWM.Domain.UserConfigs;

namespace GlazeWM.Bar.Components
{
  public class ComponentViewModel : ViewModelBase
  {
    protected readonly BarViewModel _parentViewModel;
    protected readonly BarComponentConfig _componentConfig;

    public string Background => XamlHelper.FormatXamlColor(
      _componentConfig.Background ?? _parentViewModel.Background
    );
    public string Foreground => XamlHelper.FormatXamlColor(
      _componentConfig.Foreground ?? _parentViewModel.Foreground
    );
    public string FontFamily => _componentConfig.FontFamily ?? _parentViewModel.FontFamily;
    public string FontWeight => _componentConfig.FontWeight ?? _parentViewModel.FontWeight;
    public string FontSize => _componentConfig.FontSize ?? _parentViewModel.FontSize;
    public string BorderColor => XamlHelper.FormatXamlColor(_componentConfig.BorderColor);
    public string BorderWidth => XamlHelper.ShorthandToXamlProperty(_componentConfig.BorderWidth);
    public string Padding => XamlHelper.ShorthandToXamlProperty(_componentConfig.Padding);
    public string Margin => XamlHelper.ShorthandToXamlProperty(_componentConfig.Margin);

    public ComponentViewModel(BarViewModel parentViewModel, BarComponentConfig baseConfig)
    {
      _parentViewModel = parentViewModel;
      _componentConfig = baseConfig;
    }
  }
}
