using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Reactive.Linq;
using System.Windows.Threading;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Domain.Workspaces.Events;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Bar
{
  public class BarViewModel : ViewModelBase
  {
    public ObservableCollection<Workspace> Workspaces { get; set; } = new ObservableCollection<Workspace>();
    public string Background { get; set; }
    public string FontFamily { get; set; }
    public string FontSize { get; set; }
    public string BorderColor { get; set; }
    public string BorderWidth { get; set; }
    public string Padding { get; set; }
    public double Opacity { get; set; }
    public List<BarComponentConfig> ComponentsLeft { get; set; }
    public List<BarComponentConfig> ComponentsCenter { get; set; }
    public List<BarComponentConfig> ComponentsRight { get; set; }
    public Dispatcher Dispatcher { get; set; }
    public Monitor Monitor { get; set; }
    private readonly Bus _bus;
    private readonly UserConfigService _userConfigService;

    public BarViewModel(Bus bus, UserConfigService userConfigService)
    {
      _bus = bus;
      _userConfigService = userConfigService;
    }

    public void InitializeState()
    {
      var barConfig = _userConfigService.UserConfig.Bar;
      Background = barConfig.Background;
      FontFamily = barConfig.FontFamily;
      FontSize = barConfig.FontSize;
      BorderColor = barConfig.BorderColor;
      BorderWidth = ShorthandToXamlProperty(barConfig.BorderWidth);
      Padding = ShorthandToXamlProperty(barConfig.Padding);
      Opacity = barConfig.Opacity;
      ComponentsLeft = barConfig.ComponentsLeft;
      ComponentsCenter = barConfig.ComponentsCenter;
      ComponentsRight = barConfig.ComponentsRight;

      var workspaceAttachedEvent = _bus.Events.Where(@event => @event is WorkspaceAttachedEvent);
      var workspaceDetachedEvent = _bus.Events.Where(@event => @event is WorkspaceDetachedEvent);
      var focusChangedEvent = _bus.Events.Where(@event => @event is FocusChangedEvent);

      // Refresh contents of items source.
      Observable.Merge(workspaceAttachedEvent, workspaceDetachedEvent, focusChangedEvent)
        .Subscribe(_observer => UpdateWorkspaces());

      UpdateWorkspaces();
    }

    /// <summary>
    /// Convert shorthand properties from user config (ie. `Padding`, `Margin`, and `BorderWidth`)
    /// to be compatible with their equivalent XAML properties (ie. `Padding`, `Margin`, and
    /// `BorderThickness`). Shorthand properties follow the 1-to-4 value syntax used in CSS.
    /// </summary>
    private string ShorthandToXamlProperty(string shorthand)
    {
      var shorthandParts = shorthand.Split(" ");

      return shorthandParts.Count() switch
      {
        1 => shorthand,
        2 => $"{shorthandParts[1]},{shorthandParts[0]},{shorthandParts[1]},{shorthandParts[0]}",
        3 => $"{shorthandParts[1]},{shorthandParts[0]},{shorthandParts[1]},{shorthandParts[2]}",
        4 => $"{shorthandParts[3]},{shorthandParts[0]},{shorthandParts[1]},{shorthandParts[2]}",
        _ => throw new ArgumentException(),
      };
    }

    public void UpdateWorkspaces()
    {
      Dispatcher.Invoke(() =>
      {
        Workspaces.Clear();

        foreach (var workspace in Monitor.Children)
          Workspaces.Add(workspace as Workspace);
      });
    }
  }
}
