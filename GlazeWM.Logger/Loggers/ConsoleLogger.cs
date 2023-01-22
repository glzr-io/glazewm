using System.Windows;
using System.Windows.Documents;
using System.Windows.Media;
using Microsoft.Extensions.Logging;

namespace GlazeWM.Logger.Loggers
{
  public class ConsoleLogger : ILoggerBackend
  {
    private Console.Console _consoleWin;

    public ConsoleLogger()
    {
      var uiThread = new Thread(() =>
      {
        _consoleWin = new Console.Console();
        System.Windows.Threading.Dispatcher.Run();
      });
      uiThread.SetApartmentState(ApartmentState.STA);
      uiThread.Start();

      //TODO: clean inter-thread synchronization
      while (_consoleWin == null)
      { }
    }

    public void Log<TState>(LogLevel logLevel, EventId eventId, TState state, Exception? exception, Func<TState, Exception?, string> formatter)
    {
      _consoleWin.Dispatcher.Invoke(() =>
      {
        Paragraph p =
          new Paragraph(new Run(LoggerFormatter.Format(logLevel, eventId.Name, formatter(state, exception))))
          {
            Foreground = logLevel switch
            {
              LogLevel.Information => Brushes.DodgerBlue,
              LogLevel.Warning => Brushes.Yellow,
              LogLevel.Error => Brushes.OrangeRed,
              _ => Brushes.White
            },
            Margin = new Thickness(0)
          };
        _consoleWin.LogTextBox.Document.Blocks.Add(p);
        _consoleWin.LogTextBox.ScrollToEnd();
      });
    }

    public bool IsEnabled(LogLevel logLevel)
    {
      return true;
    }

    public IDisposable BeginScope<TState>(TState state)
    {
      return null!;
    }

    public void Dispose()
    {
      _consoleWin.Dispatcher.Invoke(() =>
      {
        _consoleWin.CloseWindow = true;
        _consoleWin.Close();
      });
    }
  }
}
