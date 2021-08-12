using LarsWM.Bar.CommandHandlers;
using LarsWM.Bar.EventHandlers;
using LarsWM.Infrastructure.Bussing;
using Microsoft.Extensions.DependencyInjection;
using System;

namespace LarsWM.Bar
{
  public static class DependencyInjection
  {
    public static IServiceCollection AddBarServices(this IServiceCollection services)
    {
      services.AddSingleton<LaunchBarOnMonitorHandler>();
      services.AddSingleton<MonitorAddedHandler>();

      return services;
    }

    public static IServiceProvider RegisterBarHandlers(this IServiceProvider serviceProvider)
    {
      var bus = serviceProvider.GetRequiredService<IBus>();
      bus.RegisterCommandHandler<LaunchBarOnMonitorHandler>();
      bus.RegisterEventHandler<MonitorAddedHandler>();

      return serviceProvider;
    }
  }
}
