using System;
using System.Collections.Generic;
using System.Reactive.Linq;
using System.Windows.Threading;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Domain.Workspaces.Events;

namespace GlazeWM.Bar.Components
{
  public class MonocleIndicatorComponentViewModel : ComponentViewModel
  {
    private Dispatcher _dispatcher => _parentViewModel.Dispatcher;
    private readonly Bus _bus = ServiceLocator.GetRequiredService<Bus>();

    private readonly WorkspaceService _workspaceService =
      ServiceLocator.GetRequiredService<WorkspaceService>();
    private readonly MonocleIndicatorComponentConfig _config;

    private LabelViewModel _label;
    public LabelViewModel Label
    {
      get => _label;
      protected set => SetField(ref _label, value);
    }

    public MonocleIndicatorComponentViewModel(
      BarViewModel parentViewModel,
      MonocleIndicatorComponentConfig config) : base(parentViewModel, config)
    {
      _config = config;

      _bus.Events.Where(
        (@event) => @event is
          ExitWorkspaceMonocleEvent or
          EnterWorkspaceMonocleEvent or
          // would be better if there is a workspaceChanged event
          FocusChangedEvent
      )
        .TakeUntil(_parentViewModel.WindowClosing)
        .Subscribe((_) => _dispatcher.Invoke(() => Label = CreateLabel()));
    }

    private LabelViewModel CreateLabel()
    {
      var workspace = _workspaceService.GetFocusedWorkspace();
      var isMonocle = workspace.isMonocle;

      var label = isMonocle
        ? _config.LabelEntered
        : _config.LabelExited;

      return XamlHelper.ParseLabel(label, new Dictionary<string, Func<string>>(), this);
    }
  }
}
