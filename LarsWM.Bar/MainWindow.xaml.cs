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
using System.Windows.Data;

namespace LarsWM.Bar
{
  /// <summary>
  /// Interaction logic for MainWindow.xaml
  /// </summary>
  public partial class MainWindow : Window
  {
    private Bus _bus { get; }
    private WorkspaceService _workspaceService { get; }
    private static object _lock = new object();

    public MainWindow(Monitor monitor, WorkspaceService workspaceService, Bus bus)
    {
      _bus = bus;
      _workspaceService = workspaceService;

      InitializeComponent();

      // TODO: Bind padding, bg color, button bg color and font from user config.
      Top = monitor.Y;
      Left = monitor.X;
      Width = monitor.Width;
      // TODO: Change height to be set in XAML.
      Height = 50;

      var workspaces = new ObservableCollection<Workspace>();
      BindingOperations.EnableCollectionSynchronization(workspaces, _lock);

      WorkspaceItems.ItemsSource = workspaces;
      RefreshState(monitor);

      _bus.Events.Where(@event => @event is WorkspaceAttachedEvent).Subscribe(observer =>
      {
        // Refresh contents of items source.
        RefreshState(monitor);
      });
    }

    private void RefreshState(Monitor monitor)
    {
      (WorkspaceItems.ItemsSource as ObservableCollection<Workspace>).Clear();

      foreach (var workspace in monitor.Children)
        (WorkspaceItems.ItemsSource as ObservableCollection<Workspace>).Add(workspace as Workspace);
    }

    private void OnWorkspaceButtonClick(object sender, RoutedEventArgs e)
    {
      var button = sender as Button;
      var clickedWorkspace = button.DataContext as Workspace;

      _bus.Invoke(new FocusWorkspaceCommand(clickedWorkspace.Name));
    }
  }
}
