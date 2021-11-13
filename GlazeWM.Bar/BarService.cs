using GlazeWM.Domain.Monitors.Events;
using GlazeWM.Infrastructure.Bussing;
using System.Reactive.Linq;
using System;
using System.Threading;
using System.Linq;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Bar
{
  public class BarService
  {
    private Bus _bus;

    public BarService(Bus bus)
    {
      _bus = bus;
    }

    public void StartApp()
    {
      var thread = new Thread(() =>
      {
        var application = new App();

        // Launch the bar window on the added monitor.
        _bus.Events.Where(@event => @event is MonitorAddedEvent)
          .Subscribe((@event) =>
          {
            application.Dispatcher.Invoke(() =>
            {
              var originalFocusedHandle = GetForegroundWindow();

              var barViewModel = new BarViewModel()
              {
                Monitor = (@event as MonitorAddedEvent).AddedMonitor,
                Dispatcher = application.Dispatcher,
              };

              var barWindow = new MainWindow(barViewModel);
              barWindow.Show();

              // Reset focus to whichever window was focused before the bar window was launched.
              SetForegroundWindow(originalFocusedHandle);
            });
          });

        application.Run();
      });

      thread.Name = "GlazeWMBar";
      thread.SetApartmentState(ApartmentState.STA);
      thread.Start();
    }

    /// <summary>
    /// Convert shorthand properties from user config (ie. `Padding`, `Margin`, and `BorderWidth`)
    /// to be compatible with their equivalent XAML properties (ie. `Padding`, `Margin`, and
    /// `BorderThickness`). Shorthand properties follow the 1-to-4 value syntax used in CSS.
    /// </summary>
    public string ShorthandToXamlProperty(string shorthand)
    {
      var shorthandParts = shorthand.Split(" ");

      return shorthandParts.Count() switch
      {
        1 => shorthand,
        2 => $"{shorthandParts[1]},{shorthandParts[0]},{shorthandParts[1]},{shorthandParts[0]}",
        3 => $"{shorthandParts[1]},{shorthandParts[0]},{shorthandParts[1]},{shorthandParts[2]}",
        4 => $"{shorthandParts[3]},{shorthandParts[0]},{shorthandParts[1]},{shorthandParts[2]}",
        _ => throw new ArgumentException(),
      };
    }
  }
}
