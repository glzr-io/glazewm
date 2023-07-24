using Microsoft.Extensions.DependencyInjection;

namespace GlazeWM.Application.IpcServer
{
  public static class DependencyInjection
  {
    public static IServiceCollection AddInterprocessServices(this IServiceCollection services)
    {
      services.AddSingleton<InterprocessService>();

      return services;
    }
  }
}
