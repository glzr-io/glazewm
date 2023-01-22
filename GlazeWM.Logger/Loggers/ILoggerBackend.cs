using Microsoft.Extensions.Logging;

namespace GlazeWM.Logger.Loggers
{
  public interface ILoggerBackend : ILogger, IDisposable
  { }
}
