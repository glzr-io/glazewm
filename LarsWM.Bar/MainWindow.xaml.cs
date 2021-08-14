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
using System.ComponentModel;
using System.Collections.Generic;
using System.Diagnostics;

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
      _bus = bus;
      _workspaceService = workspaceService;

      InitializeComponent();

      // TODO: Bind padding, bg color, button bg color and font from user config.
      this.Top = monitor.Y;
      this.Left = monitor.X;
      this.Width = monitor.Width;
      // TODO: Change height to be set in XAML.
      this.Height = 50;


      this.DataContext = new BarViewModel();

      foreach (var workspace in monitor.Children)
        (this.DataContext as BarViewModel).AddWorkspace(workspace as Workspace);

      _bus.Events.Where(@event => @event is WorkspaceAttachedEvent).Subscribe(observer =>
      {
        // Refresh contents of `workspaces` collection.
        (this.DataContext as BarViewModel).ClearWorkspaces();

        foreach (var workspace in monitor.Children)
          (this.DataContext as BarViewModel).AddWorkspace(workspace as Workspace);
      });
    }

    private void OnWorkspaceButtonClick(object sender, RoutedEventArgs e)
    {
      var button = sender as Button;
      var clickedWorkspace = button.DataContext as Workspace;

      _bus.Invoke(new FocusWorkspaceCommand(clickedWorkspace.Name));
    }
  }
}
