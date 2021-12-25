using GlazeWM.Bar;
using GlazeWM.Domain;
using GlazeWM.Infrastructure;
using Microsoft.Extensions.DependencyInjection;
using System;
using System.Diagnostics;
using System.Threading;
using System.Windows.Forms;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Bootstrapper
{
  static class Program
  {
    private static string appGuid = "325d0ed7-7f60-4925-8d1b-aa287b26b218";

    /// <summary>
    ///  The main entry point for the application.
    /// </summary>
    [STAThread]
    static void Main()
    {
      Debug.WriteLine("Application started.");

      // Prevent multiple app instances using a global UUID mutex.
      using (Mutex mutex = new Mutex(false, "Global\\" + appGuid))
      {
        if (!mutex.WaitOne(0, false))
        {
          Debug.Write(
            "Application is already running. Only one instance of this application is allowed."
          );
          return;
        }

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
        Application.Run();
      }
    }
  }
}
