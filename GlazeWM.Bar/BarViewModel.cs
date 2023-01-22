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
    public string Padding => XamlHelper.FormatRectShorthand(BarConfig.Padding);
    public double Opacity => BarConfig.Opacity;

    private TextComponentViewModel _componentSeparatorLeft => new(
        this, new TextComponentConfig
        {
          Text = BarConfig.ComponentSeparators.Left
            ?? BarConfig.ComponentSeparators.Default
        }
    );

    private TextComponentViewModel _componentSeparatorCenter => new(
        this, new TextComponentConfig
        {
          Text = BarConfig.ComponentSeparators.Center
            ?? BarConfig.ComponentSeparators.Default
        }
    );

    private TextComponentViewModel _componentSeparatorRight => new(
        this, new TextComponentConfig
        {
          Text = BarConfig.ComponentSeparators.Right
            ?? BarConfig.ComponentSeparators.Default
        }
    );

    public List<ComponentViewModel> ComponentsLeft =>
      InsertComponentSeparator(
        CreateComponentViewModels(BarConfig.ComponentsLeft),
        _componentSeparatorLeft);

    public List<ComponentViewModel> ComponentsCenter =>
      InsertComponentSeparator(
        CreateComponentViewModels(BarConfig.ComponentsCenter),
        _componentSeparatorCenter);

    public List<ComponentViewModel> ComponentsRight =>
      InsertComponentSeparator(
        CreateComponentViewModels(BarConfig.ComponentsRight),
         _componentSeparatorRight);

    private static List<ComponentViewModel> InsertComponentSeparator(
      List<ComponentViewModel> componentViewModels, TextComponentViewModel componentSeparator
    )
    {
      for (var i = 1; i < componentViewModels.Count; i += 2)
      {
        componentViewModels.Insert(i, componentSeparator);
      }
      return componentViewModels;
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

    public BarViewModel(Monitor monitor, Dispatcher dispatcher, BarConfig barConfig)
    {
      Monitor = monitor;
      Dispatcher = dispatcher;
      BarConfig = barConfig;
    }
  }
}
