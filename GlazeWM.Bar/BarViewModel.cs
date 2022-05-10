using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Linq;
using System.Windows.Threading;
using GlazeWM.Bar.Common;
using GlazeWM.Bar.Components;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using Microsoft.Extensions.DependencyInjection;

namespace GlazeWM.Bar
{
  public class BarViewModel : ViewModelBase
  {
    public Dispatcher Dispatcher { get; set; }
    public Monitor Monitor { get; set; }

    private readonly UserConfigService _userConfigService = ServiceLocator.Provider.GetRequiredService<UserConfigService>();
    private BarConfig _barConfig => _userConfigService.UserConfig.Bar;

    public BarPosition Position => _barConfig.Position;
    public string Background => _barConfig.Background;
    public string Foreground => _barConfig.Foreground;
    public string FontFamily => _barConfig.FontFamily;
    public string FontSize => _barConfig.FontSize;
    public string BorderColor => _barConfig.BorderColor;
    public string BorderWidth => BarService.ShorthandToXamlProperty(_barConfig.BorderWidth);
    public string Padding => BarService.ShorthandToXamlProperty(_barConfig.Padding);
    public double Opacity => _barConfig.Opacity;

    public List<ComponentViewModel> ComponentsLeft =>
      CreateComponentViewModels(_barConfig.ComponentsLeft);

    public List<ComponentViewModel> ComponentsCenter =>
      CreateComponentViewModels(_barConfig.ComponentsCenter);

    public List<ComponentViewModel> ComponentsRight =>
      CreateComponentViewModels(_barConfig.ComponentsRight);

    private List<ComponentViewModel> CreateComponentViewModels(List<BarComponentConfig> componentConfigs)
    {
      return componentConfigs.ConvertAll<ComponentViewModel>(config => config switch
      {
        WorkspacesComponentConfig wcc => new WorkspacesComponentViewModel(this, wcc),
        ClockComponentConfig ccc => new ClockComponentViewModel(this, ccc),
        _ => throw new ArgumentOutOfRangeException(nameof(config)),
      });
    }
  }
}
