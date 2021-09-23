using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi;
using Microsoft.Extensions.DependencyInjection;

namespace GlazeWM.Infrastructure
{
  public static class DependencyInjection
  {
    public static IServiceCollection AddInfrastructureServices(this IServiceCollection services)
    {
      services.AddSingleton<Bus>();
      services.AddSingleton<WindowEventService>();
      services.AddSingleton<KeybindingService>();

      // TODO: Change WindowsApiFacade & WindowsApiService to be compatible with DI.

      return services;
    }
  }
}
