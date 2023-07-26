using System;
using System.Collections.Generic;
using Microsoft.Extensions.DependencyInjection;

namespace GlazeWM.Infrastructure
{
  public static class ServiceLocator
  {
    public static IServiceProvider Provider { private get; set; }

    public static T GetRequiredService<T>()
    {
      return Provider.GetRequiredService<T>();
    }

    public static object GetRequiredService(Type type)
    {
      return Provider.GetRequiredService(type);
    }

    public static IEnumerable<object> GetServices(Type type)
    {
      return Provider.GetServices(type);
    }

    public static (T1, T2) GetRequiredServices<T1, T2>()
    {
      var service1 = Provider.GetRequiredService<T1>();
      var service2 = Provider.GetRequiredService<T2>();
      return (service1, service2);
    }

    public static (T1, T2, T3) GetRequiredServices<T1, T2, T3>()
    {
      var service1 = Provider.GetRequiredService<T1>();
      var service2 = Provider.GetRequiredService<T2>();
      var service3 = Provider.GetRequiredService<T3>();
      return (service1, service2, service3);
    }
  }
}
