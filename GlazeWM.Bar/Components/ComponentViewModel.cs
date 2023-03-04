using GlazeWM.Bar.Common;
using GlazeWM.Domain.UserConfigs;

namespace GlazeWM.Bar.Components
{
  public abstract class ComponentViewModel : ViewModelBase
  {
    protected readonly BarViewModel _parentViewModel;
    protected readonly BarComponentConfig _componentConfig;

    public virtual string Visibility { get; set; } = "Visible";
    public string Background => XamlHelper.FormatColor(_componentConfig.Background);
    public string Foreground =>
      XamlHelper.FormatColor(_componentConfig.Foreground ?? _parentViewModel.Foreground);
    public string FontFamily =>
      _componentConfig.FontFamily ?? _parentViewModel.FontFamily;
    public string FontWeight =>
      _componentConfig.FontWeight ?? _parentViewModel.FontWeight;
    public string FontSize =>
      XamlHelper.FormatSize(_componentConfig.FontSize ?? _parentViewModel.FontSize);
    public string BorderColor => XamlHelper.FormatColor(_componentConfig.BorderColor);
    public string BorderWidth =>
      XamlHelper.FormatRectShorthand(_componentConfig.BorderWidth);
    public string BorderRadius => XamlHelper.FormatSize(_componentConfig.BorderRadius);
    public string Padding => XamlHelper.FormatRectShorthand(_componentConfig.Padding);
    public string Margin => XamlHelper.FormatRectShorthand(_componentConfig.Margin);

    protected ComponentViewModel(BarViewModel parentViewModel, BarComponentConfig baseConfig)
    {
      _parentViewModel = parentViewModel;
      _componentConfig = baseConfig;
    }
  }
}
