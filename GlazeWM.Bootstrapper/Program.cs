using GlazeWM.Bar;
using GlazeWM.Domain;
using GlazeWM.Infrastructure;
using Microsoft.Extensions.DependencyInjection;
using System;
using System.Diagnostics;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Bootstrapper
{
  static class Program
  {
    /// <summary>
    ///  The main entry point for the application.
    /// </summary>
    [STAThread]
    static void Main()
    {
      Debug.WriteLine("Application started");

      // Set the process-default DPI awareness.
      SetProcessDpiAwarenessContext(DpiAwarenessContext.Context_PerMonitorAwareV2);

      var serviceCollection = new ServiceCollection();
      serviceCollection.AddInfrastructureServices();
      serviceCollection.AddDomainServices();
      serviceCollection.AddBarServices();
      serviceCollection.AddSingleton<Startup>();

      ServiceLocator.Provider = serviceCollection.BuildServiceProvider();

      var startup = ServiceLocator.Provider.GetRequiredService<Startup>();
      startup.Init();
    }
  }
}
