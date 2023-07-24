using Microsoft.Extensions.DependencyInjection;

namespace GlazeWM.Application.IpcServer
{
  public static class DependencyInjection
  {
    public static IServiceCollection AddIpcServerServices(this IServiceCollection services)
    {
      services.AddSingleton<IpcServerManager>();

      return services;
    }
  }
}
