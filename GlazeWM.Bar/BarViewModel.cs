using System;
using System.Collections.Generic;
using System.Windows.Threading;
using GlazeWM.Bar.Common;
using GlazeWM.Bar.Components;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;

namespace GlazeWM.Bar
{
  public class BarViewModel : ViewModelBase
  {
    public Monitor Monitor { get; }
    public Dispatcher Dispatcher { get; }
    public BarConfig BarConfig { get; }

    public BarPosition Position => BarConfig.Position;
    public string Background => XamlHelper.FormatColor(BarConfig.Background);
    public string Foreground => XamlHelper.FormatColor(BarConfig.Foreground);
    public string FontFamily => BarConfig.FontFamily;
    public string FontWeight => BarConfig.FontWeight;
    public string FontSize => XamlHelper.FormatSize(BarConfig.FontSize);
    public string BorderColor => XamlHelper.FormatColor(BarConfig.BorderColor);
    public string BorderWidth => XamlHelper.FormatRectShorthand(BarConfig.BorderWidth);
    public string BorderRadius => XamlHelper.FormatRectShorthand(BarConfig.BorderRadius);
    public string Padding => XamlHelper.FormatRectShorthand(BarConfig.Padding);
    public double Opacity => BarConfig.Opacity;

    public List<ComponentViewModel> ComponentsLeft =>
      CreateComponentViewModels(BarConfig.ComponentsLeft);

    public List<ComponentViewModel> ComponentsCenter =>
      CreateComponentViewModels(BarConfig.ComponentsCenter);

    public List<ComponentViewModel> ComponentsRight =>
      CreateComponentViewModels(BarConfig.ComponentsRight);

    public BarViewModel(Monitor monitor, Dispatcher dispatcher, BarConfig barConfig)
    {
      Monitor = monitor;
      Dispatcher = dispatcher;
      BarConfig = barConfig;
    }

    private List<ComponentViewModel> CreateComponentViewModels(
      List<BarComponentConfig> componentConfigs)
    {
      return componentConfigs.ConvertAll<ComponentViewModel>(config => config switch
      {
        BatteryComponentConfig bsc => new BatteryComponentViewModel(this, bsc),
        BindingModeComponentConfig bmc => new BindingModeComponentViewModel(this, bmc),
        ClockComponentConfig ccc => new ClockComponentViewModel(this, ccc),
        TextComponentConfig tcc => new TextComponentViewModel(this, tcc),
        TilingDirectionComponentConfig tdc => new TilingDirectionComponentViewModel(this, tdc),
        WorkspacesComponentConfig wcc => new WorkspacesComponentViewModel(this, wcc),
        WindowTitleComponentConfig wtcc => new WindowTitleComponentViewModel(this, wtcc),
        _ => throw new ArgumentOutOfRangeException(nameof(config)),
      });
    }
  }
}
