using System;
using GlazeWM.Domain.Windows;
using Microsoft.Extensions.Logging;

namespace GlazeWM.Domain.Common.Utils
{
  public static class LoggerExtensions
  {
    private static readonly Action<
      ILogger,
      string,
      string,
      string,
      Exception
    > WindowEvent = LoggerMessage.Define<string, string, string>(
      LogLevel.Debug,
      new EventId(0, nameof(LogWindowEvent)),
      "{Message}: {ProcessName} {ClassName}"
  );

    /// <summary>
    /// Extension method for consistently formatting window event logs.
    /// </summary>
    public static void LogWindowEvent<T>(this ILogger<T> logger, string message, Window window)
    {
      WindowEvent(logger, message, window.ProcessName, window.ClassName, null);
    }
  }
}
