using GlazeWM.Domain.UserConfigs.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Logger.CommandHandlers;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;

namespace GlazeWM.Logger
{
  public static class DependencyInjection
  {
    public static IServiceCollection AddLoggerServices(this IServiceCollection services)
    {
      services.AddSingleton<LoggerService>();
      services.AddSingleton(typeof(ILogger<>), typeof(Logger<>));
      services.AddSingleton<ICommandHandler<ReloadLoggerBackendCommand>, ReloadLoggerBackendHandler>();

      return services;
    }
  }
}
