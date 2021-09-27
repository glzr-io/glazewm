using Microsoft.Extensions.DependencyInjection;

namespace GlazeWM.Bar
{
  public static class DependencyInjection
  {
    public static IServiceCollection AddBarServices(this IServiceCollection services)
    {
      services.AddSingleton<BarManagerService>();

      services.AddScoped<MainWindow>();
      services.AddScoped<BarViewModel>();
      services.AddScoped<WorkspacesComponentViewModel>();

      return services;
    }
  }
}
