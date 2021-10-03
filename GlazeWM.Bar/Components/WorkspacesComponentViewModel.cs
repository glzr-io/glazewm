using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Reactive.Linq;
using System.Windows.Input;
using System.Windows.Threading;
using GlazeWM.Bar.Common;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Domain.Workspaces.Commands;
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

    public string FocusedWorkspaceBackground => _config.FocusedWorkspaceBackground;
    public string FocusedWorkspaceForeground => _config.FocusedWorkspaceForeground ?? Foreground;
    public string FocusedWorkspaceBorderColor => _config.FocusedWorkspaceBorderColor;
    public string FocusedWorkspaceBorderWidth =>
      _barService.ShorthandToXamlProperty(_config.FocusedWorkspaceBorderWidth);

    public string DisplayedWorkspaceBackground => _config.DisplayedWorkspaceBackground;
    public string DisplayedWorkspaceForeground => _config.DisplayedWorkspaceForeground ?? Foreground;
    public string DisplayedWorkspaceBorderColor => _config.DisplayedWorkspaceBorderColor;
    public string DisplayedWorkspaceBorderWidth =>
      _barService.ShorthandToXamlProperty(_config.DisplayedWorkspaceBorderWidth);

    public string DefaultWorkspaceBackground => _config.DefaultWorkspaceBackground ?? Background;
    public string DefaultWorkspaceForeground => _config.DefaultWorkspaceForeground ?? Foreground;
    public string DefaultWorkspaceBorderColor => _config.DefaultWorkspaceBorderColor;
    public string DefaultWorkspaceBorderWidth =>
      _barService.ShorthandToXamlProperty(_config.DefaultWorkspaceBorderWidth);

    public ICommand FocusWorkspaceCommand => new RelayCommand<string>(FocusWorkspace);

    public WorkspacesComponentViewModel(BarViewModel parentViewModel, WorkspacesComponentConfig config)
      : base(parentViewModel, config)
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

    public void FocusWorkspace(string workspaceName)
    {
      _bus.Invoke(new FocusWorkspaceCommand(workspaceName));
    }
  }
}
