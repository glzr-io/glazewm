using GlazeWM.Domain.UserConfigs;
using GlazeWM.Logger.Loggers;

namespace GlazeWM.Logger
{
  public class LoggerService
  {
    private readonly UserConfigService _userConfigService;

    public ILoggerBackend Backend { get; private set; }

    public LoggerService(UserConfigService userConfigService)
    {
      _userConfigService = userConfigService;
      /*
       * The constructor for the LoggerService is called by the DI container
       * before the configuration is read.
       * Reading the config in the constructor would result in a null UserConfig.
       * Therefore we write into the debug console until the configuration is valid.
       */
      Backend = new DebugLogger();
    }

    private ILoggerBackend CreateLoggerFromConfig()
    {
      return _userConfigService.LoggerConfig.Output switch
      {
        LoggerOutput.Debug => new DebugLogger(),
        LoggerOutput.File => new FileLogger(Path.GetDirectoryName(_userConfigService.UserConfigPath)),
        _ => throw new ArgumentOutOfRangeException()
      };
    }

    public void LoadConfig()
    {
      Backend.Dispose();
      Backend = CreateLoggerFromConfig();
    }

    public void Stop()
    {
      Backend.Dispose();
    }
  }
}
