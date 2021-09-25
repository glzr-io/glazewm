using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Domain.Workspaces.Events;
using GlazeWM.Infrastructure.Bussing;
using System.Collections.ObjectModel;
using System.Reactive.Linq;
using System;
using System.Windows;
using System.Windows.Controls;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.UserConfigs;
using System.Windows.Interop;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Bar
{
  /// <summary>
  /// Interaction logic for MainWindow.xaml
  /// </summary>
  public partial class MainWindow : Window
  {
    private Monitor _monitor { get; }
    private Bus _bus { get; }
    private WorkspaceService _workspaceService { get; }
    private UserConfigService _userConfigService { get; }
    private ObservableCollection<Workspace> _workspaces = new ObservableCollection<Workspace>();

    public MainWindow(Monitor monitor, WorkspaceService workspaceService, Bus bus, UserConfigService userConfigService)
    {
      _monitor = monitor;
      _bus = bus;
      _workspaceService = workspaceService;
      _userConfigService = userConfigService;

      InitializeComponent();
      SourceInitialized += MainWindow_SourceInitialized;

      var barConfig = _userConfigService.UserConfig.Bar;
      var viewModel = new BarViewModel(Dispatcher, monitor, barConfig);
      viewModel.InitializeState();
      DataContext = viewModel;

      var workspaceAttachedEvent = _bus.Events.Where(@event => @event is WorkspaceAttachedEvent);
      var workspaceDetachedEvent = _bus.Events.Where(@event => @event is WorkspaceDetachedEvent);
      var focusChangedEvent = _bus.Events.Where(@event => @event is FocusChangedEvent);

      // Refresh contents of items source.
      Observable.Merge(workspaceAttachedEvent, workspaceDetachedEvent, focusChangedEvent)
        .Subscribe(_observer => viewModel.UpdateWorkspaces());
    }

    private void MainWindow_SourceInitialized(object sender, EventArgs e)
    {
      PositionWindow();
    }

    /// <summary>
    /// Position and size the WPF window manually using WinAPI. When using `PerMonitorAwareV2` DPI
    /// awareness, positioning the window with WPF bindings is ambiguous and annoying.
    /// Ref: https://github.com/dotnet/wpf/issues/4127#issuecomment-790194817
    /// </summary>
    public void PositionWindow()
    {
      var windowHandle = new WindowInteropHelper(this).Handle;

      // Since window size is set manually, need to scale up height to make window DPI responsive.
      var scaledHeight = Convert.ToInt32(_userConfigService.UserConfig.Bar.Height * _monitor.ScaleFactor);

      // The first move puts it on the correct monitor, which triggers WM_DPICHANGED.
      // The +1/-1 coerces WPF to update Top/Left/Width/Height in the second move.
      MoveWindow(windowHandle, _monitor.X + 1, _monitor.Y, _monitor.Width - 1, scaledHeight, false);
      MoveWindow(windowHandle, _monitor.X, _monitor.Y, _monitor.Width, scaledHeight, true);
    }

    private void OnWorkspaceButtonClick(object sender, RoutedEventArgs e)
    {
      var button = sender as Button;
      var clickedWorkspace = button.DataContext as Workspace;

      _bus.Invoke(new FocusWorkspaceCommand(clickedWorkspace.Name));
    }
  }
}
