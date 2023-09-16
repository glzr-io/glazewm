using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Reactive.Linq;
using System.Windows.Input;
using System.Windows.Threading;
using GlazeWM.Bar.Common;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Domain.Workspaces.Events;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Bar.Components
{
  public class WorkspacesComponentViewModel : ComponentViewModel
  {
    private Dispatcher _dispatcher => _parentViewModel.Dispatcher;
    private Monitor _monitor => _parentViewModel.Monitor;
    private WorkspacesComponentConfig _config =>
      _componentConfig as WorkspacesComponentConfig;
    private readonly Bus _bus = ServiceLocator.GetRequiredService<Bus>();
    private readonly UserConfigService _userConfigService =
      ServiceLocator.GetRequiredService<UserConfigService>();

    public ObservableCollection<Workspace> Workspaces => new(_orderedWorkspaces);

    /// <summary>
    /// Get workspaces of the current monitor sorted in the order they appear in the
    /// user's config.
    /// </summary>
    private IEnumerable<Workspace> _orderedWorkspaces => _monitor.Children
      .Cast<Workspace>()
      .OrderBy((workspace) =>
        _userConfigService.WorkspaceConfigs.FindIndex(
          (workspaceConfig) => workspaceConfig.Name == workspace.Name
        )
      );

    public string FocusedWorkspaceBackground =>
      XamlHelper.FormatColor(_config.FocusedWorkspaceBackground);
    public string FocusedWorkspaceForeground =>
      XamlHelper.FormatColor(_config.FocusedWorkspaceForeground ?? Foreground);
    public string FocusedWorkspaceBorderColor =>
      XamlHelper.FormatColor(_config.FocusedWorkspaceBorderColor);
    public string FocusedWorkspaceBorderWidth =>
      XamlHelper.FormatRectShorthand(_config.FocusedWorkspaceBorderWidth);

    public string DisplayedWorkspaceBackground =>
      XamlHelper.FormatColor(_config.DisplayedWorkspaceBackground);
    public string DisplayedWorkspaceForeground =>
      XamlHelper.FormatColor(_config.DisplayedWorkspaceForeground ?? Foreground);
    public string DisplayedWorkspaceBorderColor =>
      XamlHelper.FormatColor(_config.DisplayedWorkspaceBorderColor);
    public string DisplayedWorkspaceBorderWidth =>
      XamlHelper.FormatRectShorthand(_config.DisplayedWorkspaceBorderWidth);

    public string DefaultWorkspaceBackground =>
      XamlHelper.FormatColor(_config.DefaultWorkspaceBackground);
    public string DefaultWorkspaceForeground =>
      XamlHelper.FormatColor(_config.DefaultWorkspaceForeground ?? Foreground);
    public string DefaultWorkspaceBorderColor =>
      XamlHelper.FormatColor(_config.DefaultWorkspaceBorderColor);
    public string DefaultWorkspaceBorderWidth =>
      XamlHelper.FormatRectShorthand(_config.DefaultWorkspaceBorderWidth);

    public ICommand FocusWorkspaceCommand => new RelayCommand<string>(FocusWorkspace);

    public WorkspacesComponentViewModel(
      BarViewModel parentViewModel,
      WorkspacesComponentConfig config) : base(parentViewModel, config)
    {
      var workspacesChangedEvent = _bus.Events.Where((@event) =>
        @event is WorkspaceActivatedEvent
          or WorkspaceDeactivatedEvent
          or FocusChangedEvent
          or FocusedContainerMovedEvent
      );

      // Refresh contents of workspaces collection.
      workspacesChangedEvent
        .TakeUntil(_parentViewModel.WindowClosing)
        .Subscribe(_ =>
          _dispatcher.Invoke(() => OnPropertyChanged(nameof(Workspaces)))
        );
    }

    public void FocusWorkspace(string workspaceName)
    {
      _bus.Invoke(new FocusWorkspaceCommand(workspaceName));
      _bus.Invoke(new RedrawContainersCommand());
      _bus.Invoke(new SyncNativeFocusCommand());
    }
  }
}
