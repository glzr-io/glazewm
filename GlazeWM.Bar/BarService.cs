using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Monitors.Events;
using GlazeWM.Domain.UserConfigs.Events;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Commands;
using System.Reactive.Linq;
using System;
using System.Threading;
using System.Linq;
using System.Windows;
using System.Collections.Generic;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Bar
{
  public class BarService
  {
    private readonly Bus _bus;
    private readonly MonitorService _monitorService;
    private Application _application;
    private readonly Dictionary<string, MainWindow> _activeWindowsByDeviceName = new();

    public BarService(Bus bus, MonitorService monitorService)
    {
      _bus = bus;
      _monitorService = monitorService;
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

        var barViewModel = new BarViewModel()
        {
          Monitor = targetMonitor,
          Dispatcher = _application.Dispatcher,
        };

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

    /// <summary>
    /// Convert shorthand properties from user config (ie. `Padding`, `Margin`, and `BorderWidth`)
    /// to be compatible with their equivalent XAML properties (ie. `Padding`, `Margin`, and
    /// `BorderThickness`). Shorthand properties follow the 1-to-4 value syntax used in CSS.
    /// </summary>
    /// <exception cref="ArgumentException"></exception>
    public static string ShorthandToXamlProperty(string shorthand)
    {
      var shorthandParts = shorthand.Split(" ");

      return shorthandParts.Length switch
      {
        1 => shorthand,
        2 => $"{shorthandParts[1]},{shorthandParts[0]},{shorthandParts[1]},{shorthandParts[0]}",
        3 => $"{shorthandParts[1]},{shorthandParts[0]},{shorthandParts[1]},{shorthandParts[2]}",
        4 => $"{shorthandParts[3]},{shorthandParts[0]},{shorthandParts[1]},{shorthandParts[2]}",
        _ => throw new ArgumentException(null, nameof(shorthand)),
      };
    }
  }
}
