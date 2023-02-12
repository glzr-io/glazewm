using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.CommandHandlers;
using GlazeWM.Infrastructure.Common.Commands;
using GlazeWM.Infrastructure.Serialization;
using GlazeWM.Infrastructure.WindowsApi;
using Microsoft.Extensions.DependencyInjection;

namespace GlazeWM.Infrastructure
{
  public static class DependencyInjection
  {
    public static IServiceCollection AddInfrastructureServices(this IServiceCollection services)
    {
      services.AddSingleton<Bus>();
      services.AddSingleton<KeybindingService>();
      services.AddSingleton<WindowEventService>();
      services.AddSingleton<NetworkService>();
      services.AddSingleton<JsonService>();
      services.AddSingleton<YamlService>();

      services.AddSingleton<ICommandHandler<ExitApplicationCommand>, ExitApplicationHandler>();
      services.AddSingleton<ICommandHandler<HandleFatalExceptionCommand>, HandleFatalExceptionHandler>();
      services.AddSingleton<ICommandHandler<NoopCommand>, NoopHandler>();

      // TODO: Change WindowsApiService to be compatible with DI.

      return services;
    }
  }
}
