using System;
using System.Collections.Generic;
using System.Windows.Threading;
using GlazeWM.Bar.Common;
using GlazeWM.Bar.Components;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;

namespace GlazeWM.Bar
{
  public class BarViewModel : ViewModelBase
  {
    public Dispatcher Dispatcher { get; set; }
    public Monitor Monitor { get; set; }

    private readonly UserConfigService _userConfigService =
      ServiceLocator.GetRequiredService<UserConfigService>();
    private BarConfig _barConfig => _userConfigService.BarConfig;

    public BarPosition Position => _barConfig.Position;
    public string Background => XamlHelper.FormatColor(_barConfig.Background);
    public string Foreground => XamlHelper.FormatColor(_barConfig.Foreground);
    public string FontFamily => _barConfig.FontFamily;
    public string FontWeight => _barConfig.FontWeight;
    public string FontSize => XamlHelper.FormatSize(_barConfig.FontSize);
    public string BorderColor => XamlHelper.FormatColor(_barConfig.BorderColor);
    public string BorderWidth => XamlHelper.FormatRectShorthand(_barConfig.BorderWidth);
    public string Padding => XamlHelper.FormatRectShorthand(_barConfig.Padding);
    public double Opacity => _barConfig.Opacity;

    private TextComponentViewModel _componentSeparatorLeft => new(
        this, new TextComponentConfig
        {
          Text = _barConfig.ComponentSeparators.LabelLeft
            ?? _barConfig.ComponentSeparators.Label
        }
    );

    private TextComponentViewModel _componentSeparatorCenter => new(
        this, new TextComponentConfig
        {
          Text = _barConfig.ComponentSeparators.LabelCenter
            ?? _barConfig.ComponentSeparators.Label
        }
    );

    private TextComponentViewModel _componentSeparatorRight => new(
        this, new TextComponentConfig
        {
          Text = _barConfig.ComponentSeparators.LabelRight
            ?? _barConfig.ComponentSeparators.Label
        }
    );

    public List<ComponentViewModel> ComponentsLeft =>
      InsertComponentSeparator(
        CreateComponentViewModels(_barConfig.ComponentsLeft),
        _componentSeparatorLeft);

    public List<ComponentViewModel> ComponentsCenter =>
      InsertComponentSeparator(
        CreateComponentViewModels(_barConfig.ComponentsCenter),
        _componentSeparatorCenter);

    public List<ComponentViewModel> ComponentsRight =>
      InsertComponentSeparator(
        CreateComponentViewModels(_barConfig.ComponentsRight),
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
  }
}
