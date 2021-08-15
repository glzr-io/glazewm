using LarsWM.Domain.Monitors;
using LarsWM.Domain.Workspaces;
using LarsWM.Domain.Workspaces.Commands;
using LarsWM.Domain.Workspaces.Events;
using LarsWM.Infrastructure.Bussing;
using System.Reactive.Linq;
using System;
using System.Windows;
using System.Windows.Controls;
using System.Linq;

namespace LarsWM.Bar
{
  /// <summary>
  /// Interaction logic for MainWindow.xaml
  /// </summary>
  public partial class MainWindow : Window
  {
    private Bus _bus { get; }
    private WorkspaceService _workspaceService { get; }

    public MainWindow(Monitor monitor, WorkspaceService workspaceService, Bus bus)
    {
      InitializeComponent();

      _bus = bus;
      _workspaceService = workspaceService;

      // TODO: Bind padding, bg color, button bg color and font from user config.
      this.Top = monitor.Y;
      this.Left = monitor.X;
      this.Width = monitor.Width;
      // TODO: Change height to be set in XAML.
      this.Height = 50;

      this.DataContext = new BarViewModel(Dispatcher);

      // Initialise view model with the workspaces of the current monitor.
      this.UpdateViewModel(monitor);

      _bus.Events.Where(@event => @event is WorkspaceAttachedEvent).Subscribe(observer =>
      {
        // Refresh contents of view model.
        this.UpdateViewModel(monitor);
      });
    }

    private void UpdateViewModel(Monitor monitor)
    {

      var workspaces = monitor.Children.Select(workspace => workspace as Workspace);
      (this.DataContext as BarViewModel).SetWorkspaces(workspaces);
    }

    private void OnWorkspaceButtonClick(object sender, RoutedEventArgs e)
    {
      var button = sender as Button;
      var clickedWorkspace = button.DataContext as Workspace;

      _bus.Invoke(new FocusWorkspaceCommand(clickedWorkspace.Name));
    }
  }
}
