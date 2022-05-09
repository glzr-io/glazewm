using GlazeWM.Domain.Monitors.Events;
using GlazeWM.Infrastructure.Bussing;
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
    private Application _application;
    private readonly Dictionary<string, MainWindow> _activeWindowsByDeviceName
      = new Dictionary<string, MainWindow>();

    public BarService(Bus bus)
    {
      _bus = bus;
    }

    public void StartApp()
    {
      var thread = new Thread(() =>
      {
        _application = new Application();

        // Launch the bar window on the added monitor.
        _bus.Events.Where(@event => @event is MonitorAddedEvent)
          .Subscribe((@event) =>
          {
            _application.Dispatcher.Invoke(() =>
            {
              var addedMonitor = (@event as MonitorAddedEvent).AddedMonitor;
              var originalFocusedHandle = GetForegroundWindow();

              var barViewModel = new BarViewModel()
              {
                Monitor = addedMonitor,
                Dispatcher = _application.Dispatcher,
              };

              var barWindow = new MainWindow(barViewModel);
              barWindow.Show();

              // Store active window.
              _activeWindowsByDeviceName[addedMonitor.DeviceName] = barWindow;

              // Reset focus to whichever window was focused before the bar window was launched.
              SetForegroundWindow(originalFocusedHandle);
            });
          });

        _bus.Events.Where(@event => @event is MonitorRemovedEvent)
          .Subscribe((@event) =>
          {
            _application.Dispatcher.Invoke(() =>
            {
              var deviceName = (@event as MonitorRemovedEvent).RemovedDeviceName;

              // Kill the corresponding bar window.
              var barWindow = _activeWindowsByDeviceName.GetValueOrDefault(deviceName);
              barWindow.Close();
            });
          });

        _application.Run();
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

    /// <summary>
    /// Convert shorthand properties from user config (ie. `Padding`, `Margin`, and `BorderWidth`)
    /// to be compatible with their equivalent XAML properties (ie. `Padding`, `Margin`, and
    /// `BorderThickness`). Shorthand properties follow the 1-to-4 value syntax used in CSS.
    /// </summary>
    public static string ShorthandToXamlProperty(string shorthand)
    {
      var shorthandParts = shorthand.Split(" ");

      return shorthandParts.Length switch
      {
        1 => shorthand,
        2 => $"{shorthandParts[1]},{shorthandParts[0]},{shorthandParts[1]},{shorthandParts[0]}",
        3 => $"{shorthandParts[1]},{shorthandParts[0]},{shorthandParts[1]},{shorthandParts[2]}",
        4 => $"{shorthandParts[3]},{shorthandParts[0]},{shorthandParts[1]},{shorthandParts[2]}",
        _ => throw new ArgumentException(nameof(shorthandParts)),
      };
    }
  }
}
