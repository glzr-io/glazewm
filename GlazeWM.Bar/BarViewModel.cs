using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Reactive.Linq;
using System.Windows.Threading;
using GlazeWM.Bar.Common;
using GlazeWM.Bar.Components;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Domain.Workspaces.Events;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Bussing;
using Microsoft.Extensions.DependencyInjection;

namespace GlazeWM.Bar
{
  public class BarViewModel : ViewModelBase
  {
    public Dispatcher Dispatcher { get; set; }
    public Monitor Monitor { get; set; }
    private Bus _bus = ServiceLocator.Provider.GetRequiredService<Bus>();
    private UserConfigService _userConfigService = ServiceLocator.Provider.GetRequiredService<UserConfigService>();
    private BarConfig _barConfig => _userConfigService.UserConfig.Bar;
    public ObservableCollection<Workspace> Workspaces =>
      new ObservableCollection<Workspace>(Monitor.Children.Cast<Workspace>());
    public string Background => _barConfig.Background;
    public string FontFamily => _barConfig.FontFamily;
    public string FontSize => _barConfig.FontSize;
    public string BorderColor => _barConfig.BorderColor;
    public string BorderWidth => ShorthandToXamlProperty(_barConfig.BorderWidth);
    public string Padding => ShorthandToXamlProperty(_barConfig.Padding);
    public double Opacity => _barConfig.Opacity;

    public List<ComponentViewModel> ComponentsLeft =>
      CreateComponentViewModels(_barConfig.ComponentsLeft);

    public List<ComponentViewModel> ComponentsCenter =>
      CreateComponentViewModels(_barConfig.ComponentsCenter);

    public List<ComponentViewModel> ComponentsRight =>
      CreateComponentViewModels(_barConfig.ComponentsRight);

    public BarViewModel()
    {
      var workspacesChangedEvent = _bus.Events.Where((@event) =>
        @event is WorkspaceAttachedEvent ||
        @event is WorkspaceDetachedEvent ||
        @event is FocusChangedEvent
      );

      // Refresh contents of workspaces collection.
      workspacesChangedEvent.Subscribe((_observer) =>
      {
        Dispatcher.Invoke(() => OnPropertyChanged(nameof(Workspaces)));
      });
    }

    private List<ComponentViewModel> CreateComponentViewModels(List<BarComponentConfig> componentConfigs)
    {
      return componentConfigs.Select(config =>
      {
        ComponentViewModel viewModel = config.Type switch
        {
          "workspaces" => new WorkspacesComponentViewModel(this),
          "clock" => new ClockComponentViewModel(this),
          _ => throw new ArgumentException(),
        };

        return viewModel;
      }).ToList();
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
  }
}
