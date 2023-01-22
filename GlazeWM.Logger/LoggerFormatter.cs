using System.Diagnostics;
using Microsoft.Extensions.Logging;

namespace GlazeWM.Logger
{
  public static class LoggerFormatter
  {
    private static readonly Stopwatch Watch = Stopwatch.StartNew();

    public static string Format(LogLevel level, string type, string message)
    {
      return $"{Watch.ElapsedMilliseconds:D9}|{level.ToString()[0]}|{type}|{message}";
    }
  }
}
