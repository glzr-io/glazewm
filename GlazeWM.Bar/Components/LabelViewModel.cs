using System;
using System.Collections.Generic;
using GlazeWM.Bar.Common;

namespace GlazeWM.Bar.Components
{
  public class LabelSpan
  {
    public string Text { get; }
    public string Background { get; }
    public string Foreground { get; }
    public string FontFamily { get; }
    public string FontWeight { get; }
    public string FontSize { get; }

    public LabelSpan(
      string text,
      string background,
      string foreground,
      string fontFamily,
      string fontWeight,
      string fontSize)
    {
      Text = text;
      Background = background;
      Foreground = foreground;
      FontFamily = fontFamily;
      FontWeight = fontWeight;
      FontSize = fontSize;
    }
  }

  public class LabelViewModel : ViewModelBase
  {
    public List<LabelSpan> Spans { get; }

    public LabelViewModel(List<LabelSpan> spans)
    {
      Spans = spans;
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
