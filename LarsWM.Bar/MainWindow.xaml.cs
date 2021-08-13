using LarsWM.Domain.Monitors;
using LarsWM.Domain.Workspaces;
using LarsWM.Domain.Workspaces.Commands;
using LarsWM.Infrastructure.Bussing;
using System.Windows;
using System.Windows.Controls;

namespace LarsWM.Bar
{
  /// <summary>
  /// Interaction logic for MainWindow.xaml
  /// </summary>
  public partial class MainWindow : Window
  {
    private Bus _bus { get; }

    public MainWindow(Monitor monitor, Bus bus)
    {
      _bus = bus;

      InitializeComponent();

      this.Top = monitor.Y;
      this.Left = monitor.X;
      this.Width = monitor.Width;

      // TODO: Change height to be set in XAML.
      this.Height = 50;

      // TODO: Bind padding, bg color, button bg color and font from user config.

      var workspaces = monitor.Children;
      workspaceItems.ItemsSource = workspaces;
    }


    private void OnWorkspaceButtonClick(object sender, RoutedEventArgs e)
    {
      var button = sender as Button;
      var clickedWorkspace = button.DataContext as Workspace;

      _bus.Invoke(new FocusWorkspaceCommand(clickedWorkspace.Name));
    }
  }
}
