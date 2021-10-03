using GlazeWM.Bar.Common;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using Microsoft.Extensions.DependencyInjection;

namespace GlazeWM.Bar.Components
{
  public class ComponentViewModel : ViewModelBase
  {
    protected readonly BarViewModel _parentViewModel;
    protected readonly BarComponentConfig _componentConfig;
    protected readonly BarService _barService = ServiceLocator.Provider.GetRequiredService<BarService>();

    public string Background => _componentConfig.Background ?? _parentViewModel.Background;
    public string FontFamily => _componentConfig.FontFamily ?? _parentViewModel.FontFamily;
    public string FontSize => _componentConfig.FontSize ?? _parentViewModel.FontSize;
    public string BorderColor => _componentConfig.BorderColor;
    public string BorderWidth => _barService.ShorthandToXamlProperty(_componentConfig.BorderWidth);
    public string Padding => _barService.ShorthandToXamlProperty(_componentConfig.Padding);
    public string Margin => _barService.ShorthandToXamlProperty(_componentConfig.Margin);

    public ComponentViewModel(BarViewModel parentViewModel, BarComponentConfig baseConfig)
    {
      _parentViewModel = parentViewModel;
      _componentConfig = baseConfig;
    }
  }
}
