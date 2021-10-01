using GlazeWM.Bar.Common;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using Microsoft.Extensions.DependencyInjection;

namespace GlazeWM.Bar.Components
{
  public class ComponentViewModel : ViewModelBase
  {
    private BarService _barService = ServiceLocator.Provider.GetRequiredService<BarService>();
    protected readonly BarViewModel _parentViewModel;
    protected readonly BarComponentConfig _baseConfig;
    public string Background => _baseConfig.Background;
    public string FontFamily => _baseConfig.FontFamily;
    public string FontSize => _baseConfig.FontSize;
    public string BorderColor => _baseConfig.BorderColor;
    public string BorderWidth => _barService.ShorthandToXamlProperty(_baseConfig.BorderWidth);
    public string Padding => _barService.ShorthandToXamlProperty(_baseConfig.Padding);
    public string Margin => _barService.ShorthandToXamlProperty(_baseConfig.Margin);

    public ComponentViewModel(BarViewModel parentViewModel, BarComponentConfig baseConfig)
    {
      _parentViewModel = parentViewModel;
      _baseConfig = baseConfig;
    }
  }
}
