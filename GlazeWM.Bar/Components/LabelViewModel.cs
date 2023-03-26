using System;
using System.Collections.Generic;
using GlazeWM.Bar.Common;

namespace GlazeWM.Bar.Components
{
  public class LabelSpan
  {
    public string Background { get; set; }
    public string Foreground { get; set; }
    public string FontFamily { get; set; }
    public string FontWeight { get; set; }
    public string FontSize { get; set; }
    public string Text { get; }

    public LabelSpan(string text)
    {
      Text = text;
    }
  }

  public class LabelViewModel : ViewModelBase
  {
    public List<LabelSpan> Spans { get; }
    private readonly ComponentViewModel ComponentViewModel;

    public string Background => ComponentViewModel.Background;
    public string Foreground => ComponentViewModel.Foreground;
    public string FontFamily => ComponentViewModel.FontFamily;
    public string FontWeight => ComponentViewModel.FontWeight;
    public string FontSize => ComponentViewModel.FontSize;

    public LabelViewModel(List<LabelSpan> spans, ComponentViewModel componentViewModel)
    {
      Spans = spans;
      ComponentViewModel = componentViewModel;
    }

    public void UpdateVariables(Dictionary<string, string> labelVariables)
    {
      // TODO
      throw new NotImplementedException();
    }
  }
}

/// Usage:
/**
public class BatteryComponentViewModel : ComponentViewModel
{
  private readonly BatteryComponentConfig _config;

  private LabelViewModel _label;
  public string Label
  {
      get => _label;
      protected set => SetField(ref _label, value);
  }

  public BatteryComponentViewModel(...) : base(parentViewModel, config)
  {
    _batteryComponentConfig = config;

    Label = XamlHelper.ParseLabel(config.Label, CreateVariableDict());

    Observable
      .Interval(TimeSpan.FromSeconds(3))
      .TakeUntil(_parentViewModel.WindowClosing)
      .Subscribe(_ => {
        Label.UpdateVariables(CreateVariableDict());
        OnPropertyChanged(nameof(Label));
      });
  }
}
*/

/// Alternatively:
/**
public class BatteryComponentViewModel : ComponentViewModel
{
  private readonly BatteryComponentConfig _config;

  private LabelViewModel _label;
  public string Label
  {
      get => _label;
      protected set => SetField(ref _label, value);
  }

  public BatteryComponentViewModel(...) : base(parentViewModel, config)
  {
    _batteryComponentConfig = config;

    Label = XamlHelper.ParseLabel(config.Label, GetVariableDict());

    Observable
      .Interval(TimeSpan.FromSeconds(3))
      .TakeUntil(_parentViewModel.WindowClosing)
      .Subscribe(_ => {
        Label.UpdateVariables();
        OnPropertyChanged(nameof(Label));
      });
  }

  public Dictionary<string, Action<string>> GetVariableDict()
  {
    return new()
    {
      { "battery_level", () => GetBatteryLevel() }
    }
  }
}
*/
