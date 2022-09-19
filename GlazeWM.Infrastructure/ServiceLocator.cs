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
  }
}
