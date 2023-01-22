using Microsoft.Extensions.Logging;

namespace GlazeWM.Logger
{
  public class Logger<T> : ILogger<T>
  {
    private readonly LoggerService _loggerService;
    private readonly EventId _category;

    public Logger(LoggerService service)
    {
      _loggerService = service;
      /*
       * Forwarding the ILogger interface to the backend causes a loss of the current logger type.
       * Here, we hijack the unused EventId parameter to pass the missing information. 
       */
      _category = new EventId(0, typeof(T).Name.Split('.')[^1]);
    }

    public void Log<TState>(LogLevel logLevel, EventId eventId, TState state, Exception? exception, Func<TState, Exception?, string> formatter)
    {
      _loggerService.Backend.Log(logLevel, _category, state, exception, formatter);
    }

    public bool IsEnabled(LogLevel logLevel)
    {
      return _loggerService.Backend.IsEnabled(logLevel);
    }

    public IDisposable BeginScope<TState>(TState state)
    {
      return _loggerService.Backend.BeginScope(state);
    }
  }
}
