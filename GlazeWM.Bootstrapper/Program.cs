using GlazeWM.Bar;
using GlazeWM.Domain;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Logging;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;
using System;
using System.Diagnostics;
using System.Threading;
using System.Windows.Forms;

namespace GlazeWM.Bootstrapper
{
  internal static class Program
  {
    private const string APP_GUID = "325d0ed7-7f60-4925-8d1b-aa287b26b218";

    /// <summary>
    ///  The main entry point for the application.
    /// </summary>
    [STAThread]
    private static void Main()
    {
      Debug.WriteLine("Application started.");

      // Prevent multiple app instances using a global UUID mutex.
      using var mutex = new Mutex(false, "Global\\" + APP_GUID);
      if (!mutex.WaitOne(0, false))
      {
        Debug.Write(
          "Application is already running. Only one instance of this application is allowed."
        );
        return;
      }

      var host = CreateHost();
      ServiceLocator.Provider = host.Services;

      var startup = ServiceLocator.Provider.GetRequiredService<Startup>();
      startup.Run();
      Application.Run();
    }

    private static IHost CreateHost()
    {
      return Host.CreateDefaultBuilder()
        .ConfigureServices((_, services) =>
        {
          services.AddInfrastructureServices();
          services.AddDomainServices();
          services.AddBarServices();
          services.AddSingleton<Startup>();
        })
        .ConfigureLogging(builder =>
        {
          builder
            .AddConsole(options => options.FormatterName = "customName")
            .AddConsoleFormatter<CustomFormatter, CustomOptions>(
              options => options.CustomPrefix = " ~~~~~ "
            );
        })
        .Build();
    }
  }
}
