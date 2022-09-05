using GlazeWM.Bar;
using GlazeWM.Domain;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Logging;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Console;
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Threading;
using Microsoft.Extensions.Configuration;

namespace GlazeWM.Bootstrapper
{
  internal static class Program
  {
    private const string APP_GUID = "325d0ed7-7f60-4925-8d1b-aa287b26b218";

    /// <summary>
    ///  The main entry point for the application.
    /// </summary>
    [STAThread]
    private static void Main(string[] args)
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

      var host = CreateHost(args);
      ServiceLocator.Provider = host.Services;

      var startup = ServiceLocator.Provider.GetRequiredService<Startup>();
      startup.Run();
    }

    private static IHost CreateHost(string[] args)
    {
      return Host.CreateDefaultBuilder()
        .ConfigureAppConfiguration(appConfig =>
        {
          appConfig.AddCommandLine(args, new Dictionary<string, string>
          {
            // Map CLI argument `--config` to `UserConfigPath` configuration key.
            {"--config", "UserConfigPath"}
          });
        })
        .ConfigureServices((_, services) =>
        {
          services.AddInfrastructureServices();
          services.AddDomainServices();
          services.AddBarServices();
          services.AddSingleton<Startup>();
        })
        .ConfigureLogging(builder =>
        {
          builder.ClearProviders();
          builder.AddConsole(options => options.FormatterName = "customFormatter")
            .AddConsoleFormatter<LogFormatter, ConsoleFormatterOptions>();
        })
        .Build();
    }
  }
}
