using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Reactive.Linq;
using System.Windows.Threading;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Domain.Workspaces.Events;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Bussing;
using Microsoft.Extensions.DependencyInjection;

namespace GlazeWM.Bar.Components
{
  public class WorkspacesComponentViewModel : ComponentViewModel
  {
    private Dispatcher _dispatcher => _parentViewModel.Dispatcher;
    private Monitor _monitor => _parentViewModel.Monitor;
    private WorkspacesComponentConfig _config => _componentConfig as WorkspacesComponentConfig;
    private readonly Bus _bus = ServiceLocator.Provider.GetRequiredService<Bus>();

    public ObservableCollection<Workspace> Workspaces =>
      new ObservableCollection<Workspace>(_monitor.Children.Cast<Workspace>());

    public string FocusedWorkspaceBorderWidth => _config.FocusedWorkspaceBorderWidth;
    public string FocusedWorkspaceBorderColor => _config.FocusedWorkspaceBorderColor;
    public string FocusedWorkspaceBackground => _config.FocusedWorkspaceBackground;
    public string FocusedWorkspaceForeground => _config.FocusedWorkspaceForeground ?? Foreground;

    public string DisplayedWorkspaceBorderWidth => _config.DisplayedWorkspaceBorderWidth;
    public string DisplayedWorkspaceBorderColor => _config.DisplayedWorkspaceBorderColor;
    public string DisplayedWorkspaceBackground => _config.DisplayedWorkspaceBackground;
    public string DisplayedWorkspaceForeground => _config.DisplayedWorkspaceForeground ?? Foreground;

    public string DefaultWorkspaceBorderWidth => _config.DefaultWorkspaceBorderWidth;
    public string DefaultWorkspaceBorderColor => _config.DefaultWorkspaceBorderColor;
    public string DefaultWorkspaceBackground => _config.DefaultWorkspaceBackground ?? Background;
    public string DefaultWorkspaceForeground => _config.DefaultWorkspaceForeground ?? Foreground;

    public WorkspacesComponentViewModel(BarViewModel parentViewModel, WorkspacesComponentConfig config) : base(parentViewModel, config)
    {
      var workspacesChangedEvent = _bus.Events.Where((@event) =>
        @event is WorkspaceAttachedEvent ||
        @event is WorkspaceDetachedEvent ||
        @event is FocusChangedEvent
      );

      // Refresh contents of workspaces collection.
      workspacesChangedEvent.Subscribe((_observer) =>
      {
        _dispatcher.Invoke(() => OnPropertyChanged(nameof(Workspaces)));
      });
    }
  }
}
