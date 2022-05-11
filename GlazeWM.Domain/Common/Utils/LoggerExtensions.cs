using GlazeWM.Domain.Windows;
using Microsoft.Extensions.Logging;

namespace GlazeWM.Domain.Common.Utils
{
  public static class LoggerExtensions
  {
    /// <summary>
    /// Extension method for consistently formatting window event logs.
    /// </summary>
    public static void LogWindowEvent<T>(this ILogger<T> logger, string message, Window window)
    {
      logger.LogDebug(
        $"{message}: {{processName}} {{className}}",
        window.ProcessName,
        window.ClassName
      );
    }
  }
}
