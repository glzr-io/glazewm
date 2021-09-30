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
    private new readonly WorkspacesComponentConfig _config;
    private Bus _bus = ServiceLocator.Provider.GetRequiredService<Bus>();
    private Dispatcher _dispatcher => _parentViewModel.Dispatcher;
    private Monitor _monitor => _parentViewModel.Monitor;

    public ObservableCollection<Workspace> Workspaces =>
      new ObservableCollection<Workspace>(_monitor.Children.Cast<Workspace>());

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
