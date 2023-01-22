using Microsoft.Extensions.Logging;

namespace GlazeWM.Logger.Loggers
{
  public class FileLogger : ILoggerBackend
  {
    private const string FileName = "Logs.txt";
    private readonly StreamWriter _sw;

    public FileLogger(string dir)
    {
      _sw = new StreamWriter(File.Open($"{dir}\\{FileName}", FileMode.Append));
    }

    ~FileLogger()
    {
      Dispose(false);
    }

    public void Log<TState>(LogLevel logLevel, EventId eventId, TState state, Exception? exception, Func<TState, Exception?, string> formatter)
    {
      _sw.WriteLine(LoggerFormatter.Format(logLevel, eventId.Name!, formatter(state, exception)));
      /*
       * This is impacting performances but without it
       * we'd lose un-flushed lines in case of app crash.
       */
      _sw.Flush();
    }

    public bool IsEnabled(LogLevel logLevel)
    {
      return true;
    }

    public IDisposable BeginScope<TState>(TState state)
    {
      return null!;
    }

    private void Dispose(bool disposing)
    {
      if (disposing)
        _sw.Close();
    }

    public void Dispose()
    {
      Dispose(true);
      GC.SuppressFinalize(this);
    }
  }
}
