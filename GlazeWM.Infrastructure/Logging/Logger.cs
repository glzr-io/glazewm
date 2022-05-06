using System.Collections.Generic;

namespace GlazeWM.Infrastructure.Logging
{
  public class Logger
  {
    public string Name { get; }
    private LogLevel MinLogLevel { get; }

    private readonly List<LogLevel> LogLevelOrder = new List<LogLevel> {
      LogLevel.DEBUG,
      LogLevel.INFO,
      LogLevel.WARNING,
      LogLevel.ERROR,
    };

    public Logger(string name, LogLevel minLogLevel)
    {
      Name = name;
      MinLogLevel = minLogLevel;
    }

    public void Debug(string message)
    {
      if (ShouldLog(LogLevel.DEBUG))
        Log(message);
    }

    public void Info(string message)
    {
      if (ShouldLog(LogLevel.INFO))
        Log(message);
    }

    public void Warning(string message)
    {
      if (ShouldLog(LogLevel.WARNING))
        Log(message);
    }

    public void Error(string message)
    {
      if (ShouldLog(LogLevel.ERROR))
        Log(message);
    }

    private void Log(string message)
    {
      System.Diagnostics.Debug.WriteLine($"[{Name}] {message}");
    }

    private bool ShouldLog(LogLevel logLevel)
    {
      return LogLevelOrder.IndexOf(logLevel) >= LogLevelOrder.IndexOf(MinLogLevel);
    }
  }
}

