using Microsoft.Extensions.DependencyInjection;

namespace LarsWM.Bar
{
  public static class DependencyInjection
  {
    public static IServiceCollection AddBarServices(this IServiceCollection services)
    {
      services.AddSingleton<BarManagerService>();

      return services;
    }
  }
}
