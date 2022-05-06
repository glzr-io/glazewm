namespace GlazeWM.Infrastructure.Logging
{
  public class LoggingService
  {
    // TODO: Add some way to configure the minimum log level. For example, via CLI flag when
    // launching or an `appsettings.json` config file.
    private const LogLevel DefaultMinLogLevel = LogLevel.DEBUG;

    public static Logger CreateLogger(string name)
    {
      return new Logger(name, DefaultMinLogLevel);
    }
  }
}

