using System;
using System.Windows;
using System.Windows.Interop;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Bar
{
  /// <summary>
  /// Interaction logic for MainWindow.xaml
  /// </summary>
  public partial class MainWindow : Window
  {
    private UserConfigService _userConfigService { get; }
    private BarViewModel _barViewModel { get; }
    private Monitor _monitor => _barViewModel.Monitor;

    public MainWindow(UserConfigService userConfigService, BarViewModel barViewModel)
    {
      _userConfigService = userConfigService;
      _barViewModel = barViewModel;

      InitializeComponent();
    }

    public void BindToMonitor(Monitor monitor)
    {
      throw new NotImplementedException();
    }

    protected override void OnSourceInitialized(EventArgs e)
    {
      base.OnSourceInitialized(e);

      var windowHandle = new WindowInteropHelper(this).Handle;
      PositionWindow(windowHandle);
    }

    /// <summary>
    /// Position and size the WPF window manually using WinAPI. When using `PerMonitorAwareV2` DPI
    /// awareness, positioning the window with WPF bindings is ambiguous and annoying.
    /// Ref: https://github.com/dotnet/wpf/issues/4127#issuecomment-790194817
    /// </summary>
    private void PositionWindow(IntPtr windowHandle)
    {
      // Since window size is set manually, need to scale up height to make window DPI responsive.
      var barHeight = _userConfigService.UserConfig.Bar.Height;
      var scaledBarHeight = Convert.ToInt32(barHeight * _monitor.ScaleFactor);

      // The first move puts it on the correct monitor, which triggers WM_DPICHANGED.
      // The +1/-1 coerces WPF to update Top/Left/Width/Height in the second move.
      MoveWindow(windowHandle, _monitor.X + 1, _monitor.Y, _monitor.Width - 1, scaledBarHeight, false);
      MoveWindow(windowHandle, _monitor.X, _monitor.Y, _monitor.Width, scaledBarHeight, true);
    }
  }
}
