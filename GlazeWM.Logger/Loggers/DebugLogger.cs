using System.Diagnostics;
using Microsoft.Extensions.Logging;

namespace GlazeWM.Logger.Loggers
{
  internal class DebugLogger : ILoggerBackend
  {
    public void Log<TState>(LogLevel logLevel, EventId eventId, TState state, Exception? exception, Func<TState, Exception?, string> formatter)
    {
      Debug.WriteLine(LoggerFormatter.Format(logLevel, eventId.Name!, formatter(state, exception)));
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
    { }
  }
}
