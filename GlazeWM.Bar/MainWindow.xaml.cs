using System;
using System.Reactive.Linq;
using System.Windows;
using System.Windows.Interop;
using System.Windows.Threading;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Monitors.Events;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Utils;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Bar
{
  /// <summary>
  /// Interaction logic for MainWindow.xaml
  /// </summary>
  public partial class MainWindow : Window
  {
    private readonly Bus _bus = ServiceLocator.GetRequiredService<Bus>();
    private BarViewModel _barViewModel { get; }
    private Dispatcher _dispatcher => _barViewModel.Dispatcher;
    private Monitor _monitor => _barViewModel.Monitor;

    public MainWindow(BarViewModel barViewModel)
    {
      _barViewModel = barViewModel;
      DataContext = barViewModel;

      InitializeComponent();
    }

    protected override void OnSourceInitialized(EventArgs e)
    {
      base.OnSourceInitialized(e);

      var windowHandle = new WindowInteropHelper(this).Handle;
      HideFromTaskSwitcher(windowHandle);
      PositionWindow(windowHandle);

      // Reposition window on changes to the monitor's working area.
      _bus.Events.Where(@event => @event is WorkingAreaResizedEvent)
        .Subscribe(_ => _dispatcher.Invoke(() => PositionWindow(windowHandle)));
    }

    /// <summary>
    /// Hide the WPF window from task switcher (alt+tab menu).
    /// </summary>
    private static void HideFromTaskSwitcher(IntPtr windowHandle)
    {
      var exstyle = (int)GetWindowLongPtr(windowHandle, GWLEXSTYLE);
      exstyle |= (int)WindowStylesEx.ToolWindow;
      SetWindowLongPtr(windowHandle, GWLEXSTYLE, (IntPtr)exstyle);
    }

    /// <summary>
    /// Position and size the WPF window manually using WinAPI. When using `PerMonitorAwareV2` DPI
    /// awareness, positioning the window with WPF bindings is ambiguous and annoying.
    /// Ref: https://github.com/dotnet/wpf/issues/4127#issuecomment-790194817
    /// </summary>
    private void PositionWindow(IntPtr windowHandle)
    {
      // Since window size is set manually, need to scale up height to make window DPI responsive.
      var barHeight = UnitsHelper.TrimUnits(_barViewModel.BarConfig.Height);
      var scaledBarHeight = Convert.ToInt32(barHeight * _monitor.ScaleFactor);

      // Get offset from top of monitor.
      var barOffsetY = _barViewModel.BarConfig.Position == BarPosition.Bottom
        ? _monitor.Height - scaledBarHeight
        : 0;

      var floatBarOffsetX = UnitsHelper.TrimUnits(_barViewModel.BarConfig.OffsetX);
      var floatBarOffsetY = UnitsHelper.TrimUnits(_barViewModel.BarConfig.OffsetY);

      // The first move puts it on the correct monitor, which triggers WM_DPICHANGED.
      MoveWindow(
        windowHandle,
        _monitor.X + floatBarOffsetX,
        _monitor.Y + barOffsetY + floatBarOffsetY,
        _monitor.Width - (floatBarOffsetX * 2),
        scaledBarHeight,
        true
      );

      // The +1/-1 coerces WPF to update Top/Left/Width/Height in the second move.
      MoveWindow(
        windowHandle,
        _monitor.X + floatBarOffsetX,
        _monitor.Y + barOffsetY + floatBarOffsetY,
        _monitor.Width - (floatBarOffsetX * 2),
        scaledBarHeight,
        true
      );
    }
  }
}
