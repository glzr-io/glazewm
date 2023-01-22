using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Linq;
using System.Threading;
using System.Windows;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Monitors.Events;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.UserConfigs.Events;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Commands;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Bar
{
  public class BarService
  {
    private readonly Bus _bus;
    private readonly MonitorService _monitorService;
    private readonly UserConfigService _userConfigService;

    private Application _application;
    private readonly Dictionary<string, MainWindow> _activeWindowsByDeviceName = new();

    public BarService(
      Bus bus,
      MonitorService monitorService,
      UserConfigService userConfigService)
    {
      _bus = bus;
      _monitorService = monitorService;
      _userConfigService = userConfigService;
    }

    public void StartApp()
    {
      var thread = new Thread(() =>
      {
        try
        {
          _application = new()
          {
            ShutdownMode = ShutdownMode.OnExplicitShutdown
          };

          // Launch the bar window on the added monitor.
          _bus.Events.OfType<MonitorAddedEvent>()
            .Subscribe((@event) => ShowWindow(@event.AddedMonitor));

          _bus.Events.OfType<MonitorRemovedEvent>()
            .Subscribe((@event) => CloseWindow(@event.RemovedDeviceName));

          _bus.Events.OfType<UserConfigReloadedEvent>()
            .Subscribe((_) => RestartApp());

          _application.Run();
        }
        catch (Exception exception)
        {
          _bus.Invoke(new HandleFatalExceptionCommand(exception));
        }
      })
      {
        Name = "GlazeWMBar"
      };
      thread.SetApartmentState(ApartmentState.STA);
      thread.Start();
    }

    public void ExitApp()
    {
      _application.Dispatcher.Invoke(() => _application.Shutdown());
    }

    private void RestartApp()
    {
      foreach (var deviceName in _activeWindowsByDeviceName.Keys.ToList())
        CloseWindow(deviceName);

      foreach (var monitor in _monitorService.GetMonitors())
        ShowWindow(monitor);
    }

    public void ShowWindow(Domain.Monitors.Monitor targetMonitor)
    {
      _application.Dispatcher.Invoke(() =>
      {
        var originalFocusedHandle = GetForegroundWindow();

        var barConfig = _userConfigService.GetBarConfigForMonitor(targetMonitor);
        var barViewModel = new BarViewModel(
          targetMonitor,
          _application.Dispatcher,
          barConfig
        );

        var barWindow = new MainWindow(barViewModel);
        barWindow.Show();

        // Store active window.
        _activeWindowsByDeviceName[targetMonitor.DeviceName] = barWindow;

        // Reset focus to whichever window was focused before the bar window was launched.
        SetForegroundWindow(originalFocusedHandle);
      });
    }

    private void CloseWindow(string deviceName)
    {
      _application.Dispatcher.Invoke(() =>
      {
        // Kill the corresponding bar window.
        var barWindow = _activeWindowsByDeviceName.GetValueOrDefault(deviceName);
        barWindow.Close();
      });
    }
  }
}
