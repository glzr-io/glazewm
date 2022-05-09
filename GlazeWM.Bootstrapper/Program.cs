using GlazeWM.Bar;
using GlazeWM.Domain;
using GlazeWM.Infrastructure;
using Microsoft.Extensions.DependencyInjection;
using System;
using System.Diagnostics;
using System.Threading;
using System.Windows.Forms;

namespace GlazeWM.Bootstrapper
{
  internal static class Program
  {
    private const string appGuid = "325d0ed7-7f60-4925-8d1b-aa287b26b218";

    /// <summary>
    ///  The main entry point for the application.
    /// </summary>
    [STAThread]
    private static void Main()
    {
      Debug.WriteLine("Application started.");

      // Prevent multiple app instances using a global UUID mutex.
      using (var mutex = new Mutex(false, "Global\\" + appGuid))
      {
        if (!mutex.WaitOne(0, false))
        {
          Debug.Write(
            "Application is already running. Only one instance of this application is allowed."
          );
          return;
        }

        ServiceLocator.Provider = BuildServiceProvider();

        var startup = ServiceLocator.Provider.GetRequiredService<Startup>();
        startup.Run();
        Application.Run();
      }
    }

    private static ServiceProvider BuildServiceProvider()
    {
      var serviceCollection = new ServiceCollection();
      serviceCollection.AddInfrastructureServices();
      serviceCollection.AddDomainServices();
      serviceCollection.AddBarServices();
      serviceCollection.AddSingleton<Startup>();

      return serviceCollection.BuildServiceProvider();
    }
  }
}
