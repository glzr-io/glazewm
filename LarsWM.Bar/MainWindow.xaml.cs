using LarsWM.Domain.Monitors;
using LarsWM.Domain.Workspaces;
using LarsWM.Domain.Workspaces.Commands;
using LarsWM.Domain.Workspaces.Events;
using LarsWM.Infrastructure.Bussing;
using System.Collections.ObjectModel;
using System.Reactive.Linq;
using System;
using System.Windows;
using System.Windows.Controls;
using LarsWM.Domain.Containers.Events;
using LarsWM.Domain.UserConfigs;

namespace LarsWM.Bar
{
  /// <summary>
  /// Interaction logic for MainWindow.xaml
  /// </summary>
  public partial class MainWindow : Window
  {
    private Bus _bus { get; }
    private WorkspaceService _workspaceService { get; }
    private UserConfigService _userConfigService { get; }
    private ObservableCollection<Workspace> _workspaces = new ObservableCollection<Workspace>();

    public MainWindow(Monitor monitor, WorkspaceService workspaceService, Bus bus, UserConfigService userConfigService)
    {
      _bus = bus;
      _workspaceService = workspaceService;
      _userConfigService = userConfigService;

      InitializeComponent();

      // TODO: Bind padding, bg color, button bg color and font from user config.
      Top = monitor.Y;
      Left = monitor.X;
      Height = _userConfigService.UserConfig.Bar.Height;

      // WPF automatically changes the size of the bar based on display scale factor. This causes
      // the bar width to overflow on scale factors > 100%, so need to adjust accordingly.
      Width = Convert.ToInt32(monitor.Width / monitor.ScaleFactor);

      RefreshState(monitor);

      WorkspaceItems.ItemsSource = _workspaces;

      var workspaceAttachedEvent = _bus.Events.Where(@event => @event is WorkspaceAttachedEvent);
      var workspaceDetachedEvent = _bus.Events.Where(@event => @event is WorkspaceDetachedEvent);
      var focusChangedEvent = _bus.Events.Where(@event => @event is FocusChangedEvent);

      // Refresh contents of items source.
      Observable.Merge(workspaceAttachedEvent, workspaceDetachedEvent, focusChangedEvent)
        .Subscribe(_observer => Dispatcher.Invoke(() => RefreshState(monitor)));
    }

    private void RefreshState(Monitor monitor)
    {
      _workspaces.Clear();

      foreach (var workspace in monitor.Children)
        _workspaces.Add(workspace as Workspace);
    }

    private void OnWorkspaceButtonClick(object sender, RoutedEventArgs e)
    {
      var button = sender as Button;
      var clickedWorkspace = button.DataContext as Workspace;

      _bus.Invoke(new FocusWorkspaceCommand(clickedWorkspace.Name));
    }
  }
}
