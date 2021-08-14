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
    private static object _lock = new object();
    private WorkspaceService _workspaceService { get; }

    public MainWindow(Monitor monitor, WorkspaceService workspaceService, Bus bus)
    {
      _bus = bus;

      InitializeComponent();

      // TODO: Bind padding, bg color, button bg color and font from user config.
      this.Top = monitor.Y;
      this.Left = monitor.X;
      this.Width = monitor.Width;
      // TODO: Change height to be set in XAML.
      this.Height = 50;

      var workspaces = new ObservableCollection<Workspace>();
      BindingOperations.EnableCollectionSynchronization(workspaces, _lock);

      foreach (var workspace in monitor.Children)
        workspaces.Add(workspace as Workspace);

      workspaceItems.ItemsSource = workspaces;

      _bus.Events.Where(@event => @event is WorkspaceAttachedEvent).Subscribe(observer =>
          {
            // App.Current.Dispatcher.Invoke((Action)delegate
            // {
            //   workspaces.Clear();
            //   foreach (var workspace in monitor.Children)
            //     workspaces.Add(workspace as Workspace);
            // });

            // UiContext.Send((x) =>
            // {
            //   workspaces.Clear();
            //   foreach (var workspace in monitor.Children)
            //     workspaces.Add(workspace as Workspace);
            // }, null);

            workspaces.Clear();
            foreach (var workspace in monitor.Children)
              workspaces.Add(workspace as Workspace);

            CollectionViewSource.GetDefaultView(workspaces).Refresh();
          }
      );
    }

    private void OnWorkspaceButtonClick(object sender, RoutedEventArgs e)
    {
      var button = sender as Button;
      var clickedWorkspace = button.DataContext as Workspace;

      _bus.Invoke(new FocusWorkspaceCommand(clickedWorkspace.Name));
    }
  }
}
